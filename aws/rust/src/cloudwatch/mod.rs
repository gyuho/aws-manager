use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::errors::{self, Error, Result};
use aws_sdk_cloudwatch::{types::MetricDatum, Client as MetricsClient};
use aws_sdk_cloudwatchlogs::{
    operation::{create_log_group::CreateLogGroupError, delete_log_group::DeleteLogGroupError},
    Client as LogsClient,
};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_types::SdkConfig as AwsSdkConfig;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

/// TODO: bump up to 1,000
/// ref. https://aws.amazon.com/about-aws/whats-new/2022/08/amazon-cloudwatch-metrics-increases-throughput/
const BATCH_SIZE: usize = 950;

/// Implements AWS CloudWatch manager.
#[derive(Debug, Clone)]
pub struct Manager {
    pub region: String,
    metrics_cli: MetricsClient,
    logs_cli: LogsClient,
}

impl Manager {
    pub fn new(shared_config: &AwsSdkConfig) -> Self {
        let metrics_cli = MetricsClient::new(shared_config);
        let logs_cli = LogsClient::new(shared_config);
        Self {
            region: shared_config.region().unwrap().to_string(),
            metrics_cli,
            logs_cli,
        }
    }

    pub fn metrics_client(&self) -> MetricsClient {
        self.metrics_cli.clone()
    }

    pub fn logs_client(&self) -> LogsClient {
        self.logs_cli.clone()
    }

    /// Posts CloudWatch metrics.
    ///
    /// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricData.html
    /// ref. https://docs.rs/aws-sdk-cloudwatch/latest/aws_sdk_cloudwatch/struct.Client.html#method.put_metric_data
    ///
    /// Can batch up to 1,000 data point.
    /// ref. https://aws.amazon.com/about-aws/whats-new/2022/08/amazon-cloudwatch-metrics-increases-throughput/
    ///
    /// "If a single piece of data must be accessible from more than one task
    /// concurrently, then it must be shared using synchronization primitives such as Arc."
    /// ref. https://tokio.rs/tokio/tutorial/spawning
    pub async fn put_metric_data(&self, namespace: &str, data: Vec<MetricDatum>) -> Result<()> {
        let n = data.len();
        log::info!(
            "posting CloudWatch {} metrics in the namespace '{namespace}', region '{}'",
            n,
            self.region
        );
        if n <= BATCH_SIZE {
            let ret = self
                .metrics_cli
                .put_metric_data()
                .namespace(namespace.to_string())
                .set_metric_data(Some(data))
                .send()
                .await;
            match ret {
                Ok(_) => {
                    log::info!("successfully post metrics");
                }
                Err(e) => {
                    return Err(Error::API {
                        message: format!("failed put_metric_data {:?}", e),
                        retryable: errors::is_sdk_err_retryable(&e)
                            || is_err_retryable_put_metrics_data(&e),
                    });
                }
            };
        } else {
            log::warn!(
                "put_metric_data limit is {}, got {}; batching by {}...",
                BATCH_SIZE,
                n,
                BATCH_SIZE
            );
            for batch in data.chunks(BATCH_SIZE) {
                let batch_n = batch.len();
                let ret = self
                    .metrics_cli
                    .put_metric_data()
                    .namespace(namespace.to_string())
                    .set_metric_data(Some(batch.to_vec()))
                    .send()
                    .await;
                match ret {
                    Ok(_) => {
                        log::info!("successfully post {} metrics in batch", batch_n);
                    }
                    Err(e) => {
                        return Err(Error::API {
                            message: format!("failed put_metric_data {:?}", e),
                            retryable: errors::is_sdk_err_retryable(&e)
                                || is_err_retryable_put_metrics_data(&e),
                        });
                    }
                }
                sleep(Duration::from_secs(1)).await;
            }
        }

        Ok(())
    }

    /// Creates a CloudWatch log group.
    /// ref. https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-logs-loggroup.html
    pub async fn create_log_group(&self, log_group_name: &str) -> Result<()> {
        log::info!(
            "creating CloudWatch log group '{log_group_name}' in region '{}'",
            self.region
        );
        let ret = self
            .logs_cli
            .create_log_group()
            .log_group_name(log_group_name)
            .send()
            .await;
        let already_created = match ret {
            Ok(_) => false,
            Err(e) => {
                if !is_err_already_exists_create_log_group(&e) {
                    return Err(Error::API {
                        message: format!("failed create_log_group {:?}", e),
                        retryable: errors::is_sdk_err_retryable(&e)
                            || is_err_retryable_create_log_group(&e),
                    });
                }
                log::warn!("log_group already exists ({})", e);
                true
            }
        };
        if !already_created {
            log::info!("created CloudWatch log group");
        }
        Ok(())
    }

    /// Deletes a CloudWatch log group.
    /// ref. https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-logs-loggroup.html
    pub async fn delete_log_group(&self, log_group_name: &str) -> Result<()> {
        log::info!(
            "deleting CloudWatch log group '{log_group_name}' in region '{}'",
            self.region
        );
        let ret = self
            .logs_cli
            .delete_log_group()
            .log_group_name(log_group_name)
            .send()
            .await;
        let deleted = match ret {
            Ok(_) => true,
            Err(e) => {
                let mut ignore_err: bool = false;
                if is_err_does_not_exist_delete_log_group(&e) {
                    log::warn!(
                        "delete_log_group failed; '{}' does not exist ({}",
                        log_group_name,
                        e
                    );
                    ignore_err = true
                }
                if !ignore_err {
                    return Err(Error::API {
                        message: format!("failed delete_log_group {:?}", e),
                        retryable: errors::is_sdk_err_retryable(&e)
                            || is_err_retryable_create_log_group(&e),
                    });
                }
                false
            }
        };
        if deleted {
            log::info!("deleted CloudWatch log group");
        };
        Ok(())
    }
}

#[inline]
fn is_err_retryable_put_metrics_data<E, R>(e: &SdkError<E, R>) -> bool {
    match e {
        SdkError::TimeoutError(_) | SdkError::ResponseError { .. } => true,
        SdkError::DispatchFailure(e) => e.is_timeout() || e.is_io(),
        _ => false,
    }
}

#[inline]
fn is_err_retryable_create_log_group<E, R>(e: &SdkError<E, R>) -> bool {
    match e {
        SdkError::TimeoutError(_) | SdkError::ResponseError { .. } => true,
        SdkError::DispatchFailure(e) => e.is_timeout() || e.is_io(),
        _ => false,
    }
}

#[inline]
fn is_err_already_exists_create_log_group(
    e: &SdkError<CreateLogGroupError, aws_smithy_runtime_api::client::orchestrator::HttpResponse>,
) -> bool {
    match e {
        SdkError::ServiceError(err) => err.err().is_resource_already_exists_exception(),
        _ => false,
    }
}

#[inline]
fn is_err_does_not_exist_delete_log_group(
    e: &SdkError<DeleteLogGroupError, aws_smithy_runtime_api::client::orchestrator::HttpResponse>,
) -> bool {
    match e {
        SdkError::ServiceError(err) => err.err().is_resource_not_found_exception(),
        _ => false,
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
pub const DEFAULT_CONFIG_FILE_PATH: &str = "/opt/aws/amazon-cloudwatch-agent/bin/config.json";

/// Default 10-minute
pub const DEFAULT_METRICS_COLLECTION_INTERVAL: u32 = 600;

pub const DEFAULT_LOGFILE: &str =
    "/opt/aws/amazon-cloudwatch-agent/logs/amazon-cloudwatch-agent.log";

/// Represents CloudWatch configuration.
/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
/// ref. https://serde.rs/container-attrs.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<Agent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Logs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Metrics>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Agent {
    pub metrics_collection_interval: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub logfile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
            region: None,
            logfile: String::from(DEFAULT_LOGFILE),
            debug: Some(false),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Logs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_collected: Option<LogsCollected>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_flush_interval: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct LogsCollected {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Files>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Files {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collect_list: Option<Vec<Collect>>,
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Collect {
    /// Specifies what to use as the log group name in CloudWatch Logs.
    pub log_group_name: String,
    pub log_stream_name: String,
    /// Specifies the path of the log file to upload to CloudWatch Logs.
    pub file_path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_format: Option<String>,

    /// The valid values are UTC and Local.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_removal: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_in_days: Option<u16>,
}

impl Default for Collect {
    fn default() -> Self {
        Self {
            log_group_name: String::from(""),
            log_stream_name: String::from(""),
            file_path: String::from(""),
            timestamp_format: None,
            timezone: None,
            auto_removal: None,
            retention_in_days: None,
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Metrics {
    pub namespace: String,
    pub metrics_collected: MetricsCollected,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub append_dimensions: Option<HashMap<String, String>>,

    /// Specifies the dimensions that collected metrics are to be aggregated on.
    /// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_dimensions: Option<Vec<Vec<String>>>,

    pub force_flush_interval: u32,
}

impl Default for Metrics {
    fn default() -> Self {
        // ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
        let mut m = HashMap::new();
        m.insert("InstanceId".to_string(), "${aws:InstanceId}".to_string());
        m.insert(
            "InstanceType".to_string(),
            "${aws:InstanceType}".to_string(),
        );
        m.insert(
            "AutoScalingGroupName".to_string(),
            "${aws:AutoScalingGroupName}".to_string(),
        );
        Self {
            namespace: String::new(),
            metrics_collected: MetricsCollected::default(),
            append_dimensions: Some(m),
            aggregation_dimensions: Some(vec![
                vec!["AutoScalingGroupName".to_string()],
                vec!["InstanceId".to_string(), "InstanceType".to_string()],
            ]),
            force_flush_interval: 30,
        }
    }
}

impl Metrics {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collected: MetricsCollected::new(metrics_collection_interval_seconds),
            ..Default::default()
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MetricsCollected {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Cpu>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem: Option<Mem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<Disk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diskio: Option<DiskIo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net: Option<Net>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub netstat: Option<Netstat>,
}

impl Default for MetricsCollected {
    fn default() -> Self {
        Self {
            cpu: Some(Cpu::default()),
            mem: Some(Mem::default()),
            disk: Some(Disk::default()),
            diskio: Some(DiskIo::default()),
            net: Some(Net::default()),
            netstat: Some(Netstat::default()),
        }
    }
}

impl MetricsCollected {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            cpu: Some(Cpu::new(metrics_collection_interval_seconds)),
            mem: Some(Mem::new(metrics_collection_interval_seconds)),
            disk: Some(Disk::new(metrics_collection_interval_seconds)),
            diskio: Some(DiskIo::new(metrics_collection_interval_seconds)),
            net: Some(Net::new(metrics_collection_interval_seconds)),
            netstat: Some(Netstat::new(metrics_collection_interval_seconds)),
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Cpu {
    pub resources: Vec<String>,
    pub measurement: Vec<String>,
    pub metrics_collection_interval: u32,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            resources: vec!["*".to_string()],
            measurement: vec![
                "usage_active".to_string(), // cpu_usage_* metrics is Percent
                "usage_system".to_string(), // cpu_usage_* metrics is Percent
            ],
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
        }
    }
}

impl Cpu {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Mem {
    pub measurement: Vec<String>,
    pub metrics_collection_interval: u32,
}

impl Default for Mem {
    fn default() -> Self {
        Self {
            measurement: vec!["mem_used".to_string(), "mem_total".to_string()],
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
        }
    }
}

impl Mem {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Disk {
    pub resources: Vec<String>,
    pub measurement: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_file_system_types: Option<Vec<String>>,

    pub metrics_collection_interval: u32,
}

impl Default for Disk {
    fn default() -> Self {
        Self {
            resources: vec!["/".to_string()],
            measurement: vec![
                "used".to_string(),
                "total".to_string(),
                "inodes_used".to_string(),
                "inodes_total".to_string(),
            ],
            ignore_file_system_types: Some(vec!["sysfs".to_string(), "devtmpfs".to_string()]),
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
        }
    }
}

impl Disk {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }

    pub fn new_with_resources(
        resources: Vec<String>,
        metrics_collection_interval_seconds: u32,
    ) -> Self {
        Self {
            resources,
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DiskIo {
    pub resources: Vec<String>,
    pub measurement: Vec<String>,
    pub metrics_collection_interval: u32,
}

impl Default for DiskIo {
    fn default() -> Self {
        Self {
            // "nvme0n1" for boot volume (AWS)
            // "nvme0n1p1" for boot volume (AWS)
            // "nvme1n1" for mounted EBS (AWS)
            // (run "lsblk" to find out which devices)
            resources: vec!["nvme1n1".to_string()],
            measurement: vec![
                "reads".to_string(),
                "writes".to_string(),
                "read_time".to_string(),
                "write_time".to_string(),
            ],
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
        }
    }
}

impl DiskIo {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Net {
    pub resources: Vec<String>,
    pub measurement: Vec<String>,
    pub metrics_collection_interval: u32,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            resources: vec!["*".to_string()],
            measurement: vec![
                "bytes_sent".to_string(),
                "bytes_recv".to_string(),
                "packets_sent".to_string(),
                "packets_recv".to_string(),
            ],
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
        }
    }
}

impl Net {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }
}

/// ref. https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Agent-Configuration-File-Details.html
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Netstat {
    pub measurement: Vec<String>,
    pub metrics_collection_interval: u32,
}

impl Default for Netstat {
    fn default() -> Self {
        Self {
            measurement: vec!["tcp_listen".to_string(), "tcp_established".to_string()],
            metrics_collection_interval: DEFAULT_METRICS_COLLECTION_INTERVAL,
        }
    }
}

impl Netstat {
    pub fn new(metrics_collection_interval_seconds: u32) -> Self {
        Self {
            metrics_collection_interval: metrics_collection_interval_seconds,
            ..Default::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            agent: None,
            logs: None,
            metrics: None,
        }
    }

    pub fn default() -> Self {
        let mut config = Self::new();
        config.agent = Some(Agent::default());

        // DO NOT SET THIS SINCE NAMESPACE IS MISSING
        // OTHERWISE
        // "Under path : /metrics/namespace | Error : String length must be greater than or equal to 1"
        // config.metrics = Some(Metrics::default());

        config
    }

    /// Converts to string.
    pub fn encode_json(&self) -> io::Result<String> {
        match serde_json::to_string(&self) {
            Ok(s) => Ok(s),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("failed to serialize Config to YAML {}", e),
            )),
        }
    }

    /// Saves the current configuration to disk
    /// and overwrites the file.
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing CloudWatch config to '{}'", file_path);
        let path = Path::new(file_path);
        let parent_dir = path.parent().unwrap();
        fs::create_dir_all(parent_dir)?;

        let ret = serde_json::to_vec(self);
        let d = match ret {
            Ok(d) => d,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to serialize Config to YAML {}", e),
                ));
            }
        };
        let mut f = File::create(file_path)?;
        f.write_all(&d)?;

        log::info!("successfully synced CloudWatch config to '{}'", file_path);
        Ok(())
    }

    pub fn load(file_path: &str) -> io::Result<Self> {
        log::info!("loading CloudWatch config from {}", file_path);

        if !Path::new(file_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("file {} does not exists", file_path),
            ));
        }

        let f = File::open(&file_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to open {} ({})", file_path, e),
            )
        })?;
        serde_json::from_reader(f).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("invalid JSON: {}", e))
        })
    }

    /// Validates the configuration.
    pub fn validate(&self) -> io::Result<()> {
        log::info!("validating the CloudWatch configuration");

        Ok(())
    }
}

#[test]
fn test_config() {
    use std::fs;
    let _ = env_logger::builder().is_test(true).try_init();

    let config = Config::default();
    let ret = config.encode_json();
    assert!(ret.is_ok());
    let s = ret.unwrap();
    log::info!("config: {}", s);

    let p = random_manager::tmp_path(10, Some(".json")).unwrap();
    let ret = config.sync(&p);
    assert!(ret.is_ok());
    fs::remove_file(p).unwrap();
}
