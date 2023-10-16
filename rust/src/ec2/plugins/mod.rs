pub mod scripts;

use std::{
    collections::HashSet,
    io::{self, Error, ErrorKind},
    str::FromStr,
};

use crate::ec2;
use serde::{Deserialize, Serialize};

/// Defines the Arch type.
#[derive(
    Deserialize, Serialize, std::clone::Clone, std::cmp::Eq, std::fmt::Debug, std::hash::Hash,
)]
pub enum Plugin {
    #[serde(rename = "imds")]
    Imds,
    #[serde(rename = "provider-id")]
    ProviderId,
    #[serde(rename = "vercmp")]
    Vercmp,
    #[serde(rename = "setup-local-disks")]
    SetupLocalDisks,
    #[serde(rename = "mount-bpf-fs")]
    MountBpfFs,

    #[serde(rename = "time-sync")]
    TimeSync,
    #[serde(rename = "system-limit-bump")]
    SystemLimitBump,
    #[serde(rename = "aws-cli")]
    AwsCli,
    #[serde(rename = "ssm-agent")]
    SsmAgent,
    #[serde(rename = "cloudwatch-agent")]
    CloudwatchAgent,

    #[serde(rename = "static-volume-provisioner")]
    StaticVolumeProvisioner,
    #[serde(rename = "static-ip-provisioner")]
    StaticIpProvisioner,

    #[serde(rename = "anaconda")]
    Anaconda,
    #[serde(rename = "python")]
    Python,

    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "go")]
    Go,

    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "containerd")]
    Containerd,
    #[serde(rename = "runc")]
    Runc,
    #[serde(rename = "cni-plugins")]
    CniPlugins,

    #[serde(rename = "aws-cfn-helper")]
    AwsCfnHelper,
    #[serde(rename = "saml2aws")]
    Saml2Aws,
    #[serde(rename = "aws-iam-authenticator")]
    AwsIamAuthenticator,
    #[serde(rename = "ecr-credential-helper")]
    EcrCredentialHelper,
    #[serde(rename = "ecr-credential-provider")]
    EcrCredentialProvider,

    #[serde(rename = "kubelet")]
    Kubelet,
    #[serde(rename = "kubectl")]
    Kubectl,
    #[serde(rename = "helm")]
    Helm,
    #[serde(rename = "terraform")]
    Terraform,

    #[serde(rename = "ssh-key-with-email")]
    SshKeyWithEmail,

    /// ref. <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/enhanced-networking-ena.html#ena-requirements>
    /// ref. <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/enhanced-networking-ena.html>
    /// ref. <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/enhanced-networking-ena.html#enhanced-networking-ena-ubuntu>
    #[serde(rename = "ena")]
    Ena,

    #[serde(rename = "nvidia-driver")]
    NvidiaDriver,
    #[serde(rename = "nvidia-cuda-toolkit")]
    NvidiaCudaToolkit,
    #[serde(rename = "nvidia-container-toolkit")]
    NvidiaContainerToolkit,

    #[serde(rename = "amd-radeon-gpu-driver")]
    AmdRadeonGpuDriver,

    #[serde(rename = "protobuf-compiler")]
    ProtobufCompiler,
    #[serde(rename = "cmake")]
    Cmake,
    #[serde(rename = "gcc7")]
    Gcc7,

    #[serde(rename = "dev-bark")]
    DevBark,
    #[serde(rename = "dev-faiss-gpu")]
    DevFaissGpu,

    #[serde(rename = "eks-worker-node-ami-scratch")]
    EksWorkerNodeAmiScratch,
    #[serde(rename = "eks-worker-node-ami-reuse")]
    EksWorkerNodeAmiReuse,

    #[serde(rename = "ami-info")]
    AmiInfo,
    #[serde(rename = "cluster-info")]
    ClusterInfo,

    #[serde(rename = "post-init-script")]
    PostInitScript,

    #[serde(rename = "cleanup-image-packages")]
    CleanupImagePackages,
    #[serde(rename = "cleanup-image-tmp-dir")]
    CleanupImageTmpDir,
    #[serde(rename = "cleanup-image-aws-credentials")]
    CleanupImageAwsCredentials,
    #[serde(rename = "cleanup-image-ssh-keys")]
    CleanupImageSshKeys,

    Unknown(String),
}

impl std::convert::From<&str> for Plugin {
    fn from(s: &str) -> Self {
        match s {
            "imds" => Plugin::Imds,
            "provider-id" => Plugin::ProviderId,
            "vercmp" => Plugin::Vercmp,
            "setup-local-disks" => Plugin::SetupLocalDisks,
            "mount-bpf-fs" => Plugin::MountBpfFs,
            "time-sync" => Plugin::TimeSync,
            "system-limit-bump" => Plugin::SystemLimitBump,
            "aws-cli" => Plugin::AwsCli,
            "ssm-agent" => Plugin::SsmAgent,
            "cloudwatch-agent" => Plugin::CloudwatchAgent,
            "static-volume-provisioner" => Plugin::StaticVolumeProvisioner,
            "static-ip-provisioner" => Plugin::StaticIpProvisioner,
            "anaconda" => Plugin::Anaconda,
            "python" => Plugin::Python,
            "rust" => Plugin::Rust,
            "go" => Plugin::Go,
            "docker" => Plugin::Docker,
            "containerd" => Plugin::Containerd,
            "runc" => Plugin::Runc,
            "cni-plugins" => Plugin::CniPlugins,
            "aws-cfn-helper" => Plugin::AwsCfnHelper,
            "saml2aws" => Plugin::Saml2Aws,
            "aws-iam-authenticator" => Plugin::AwsIamAuthenticator,
            "ecr-credential-helper" => Plugin::EcrCredentialHelper,
            "ecr-credential-provider" => Plugin::EcrCredentialProvider,
            "kubelet" => Plugin::Kubelet,
            "kubectl" => Plugin::Kubectl,
            "helm" => Plugin::Helm,
            "terraform" => Plugin::Terraform,
            "ssh-key-with-email" => Plugin::SshKeyWithEmail,
            "ena" => Plugin::Ena,
            "nvidia-driver" => Plugin::NvidiaDriver,
            "nvidia-cuda-toolkit" => Plugin::NvidiaCudaToolkit,
            "nvidia-container-toolkit" => Plugin::NvidiaContainerToolkit,
            "amd-radeon-gpu-driver" => Plugin::AmdRadeonGpuDriver,
            "protobuf-compiler" => Plugin::ProtobufCompiler,
            "cmake" => Plugin::Cmake,
            "gcc7" => Plugin::Gcc7,
            "dev-bark" => Plugin::DevBark,
            "dev-faiss-gpu" => Plugin::DevFaissGpu,
            "eks-worker-node-ami-scratch" => Plugin::EksWorkerNodeAmiScratch,
            "eks-worker-node-ami-reuse" => Plugin::EksWorkerNodeAmiReuse,
            "ami-info" => Plugin::AmiInfo,
            "cluster-info" => Plugin::ClusterInfo,
            "post-init-script" => Plugin::PostInitScript,
            "cleanup-image-packages" => Plugin::CleanupImagePackages,
            "cleanup-image-tmp-dir" => Plugin::CleanupImageTmpDir,
            "cleanup-image-aws-credentials" => Plugin::CleanupImageAwsCredentials,
            "cleanup-image-ssh-keys" => Plugin::CleanupImageSshKeys,
            other => Plugin::Unknown(other.to_owned()),
        }
    }
}

impl std::str::FromStr for Plugin {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Plugin::from(s))
    }
}

impl Plugin {
    /// Returns the `&str` value of the enum member.
    pub fn as_str(&self) -> &str {
        match self {
            Plugin::Imds => "imds",
            Plugin::ProviderId => "provider-id",
            Plugin::Vercmp => "vercmp",
            Plugin::SetupLocalDisks => "setup-local-disks",
            Plugin::MountBpfFs => "mount-bpf-fs",
            Plugin::TimeSync => "time-sync",
            Plugin::SystemLimitBump => "system-limit-bump",
            Plugin::AwsCli => "aws-cli",
            Plugin::SsmAgent => "ssm-agent",
            Plugin::CloudwatchAgent => "cloudwatch-agent",
            Plugin::StaticVolumeProvisioner => "static-volume-provisioner",
            Plugin::StaticIpProvisioner => "static-ip-provisioner",
            Plugin::Anaconda => "anaconda",
            Plugin::Python => "python",
            Plugin::Rust => "rust",
            Plugin::Go => "go",
            Plugin::Docker => "docker",
            Plugin::Containerd => "containerd",
            Plugin::Runc => "runc",
            Plugin::CniPlugins => "cni-plugins",
            Plugin::AwsCfnHelper => "aws-cfn-helper",
            Plugin::Saml2Aws => "saml2aws",
            Plugin::AwsIamAuthenticator => "aws-iam-authenticator",
            Plugin::EcrCredentialHelper => "ecr-credential-helper",
            Plugin::EcrCredentialProvider => "ecr-credential-provider",
            Plugin::Kubelet => "kubelet",
            Plugin::Kubectl => "kubectl",
            Plugin::Helm => "helm",
            Plugin::Terraform => "terraform",
            Plugin::SshKeyWithEmail => "ssh-key-with-email",
            Plugin::Ena => "ena",
            Plugin::NvidiaDriver => "nvidia-driver",
            Plugin::NvidiaCudaToolkit => "nvidia-cuda-toolkit",
            Plugin::NvidiaContainerToolkit => "nvidia-container-toolkit",
            Plugin::AmdRadeonGpuDriver => "amd-radeon-gpu-driver",
            Plugin::ProtobufCompiler => "protobuf-compiler",
            Plugin::Cmake => "cmake",
            Plugin::Gcc7 => "gcc7",
            Plugin::DevBark => "dev-bark",
            Plugin::DevFaissGpu => "dev-faiss-gpu",
            Plugin::EksWorkerNodeAmiScratch => "eks-worker-node-ami-scratch",
            Plugin::EksWorkerNodeAmiReuse => "eks-worker-node-ami-reuse",
            Plugin::AmiInfo => "ami-info",
            Plugin::ClusterInfo => "cluster-info",
            Plugin::PostInitScript => "post-init-script",
            Plugin::CleanupImagePackages => "cleanup-image-packages",
            Plugin::CleanupImageTmpDir => "cleanup-image-tmp-dir",
            Plugin::CleanupImageAwsCredentials => "cleanup-image-aws-credentials",
            Plugin::CleanupImageSshKeys => "cleanup-image-ssh-keys",
            Plugin::Unknown(s) => s.as_ref(),
        }
    }

    /// Returns the ranking in the sort.
    /// Must in order; volume mount first + ip provision
    /// useful when we create a base instance for AMI.
    pub fn rank(&self) -> u32 {
        match self {
            Plugin::Imds => 0,
            Plugin::ProviderId => 1,
            Plugin::Vercmp => 2,
            Plugin::SetupLocalDisks => 3,
            Plugin::MountBpfFs => 4,
            Plugin::TimeSync => 5,
            Plugin::SystemLimitBump => 6,
            Plugin::AwsCli => 7,
            Plugin::SsmAgent => 8,
            Plugin::CloudwatchAgent => 9,

            Plugin::StaticVolumeProvisioner => 20,
            Plugin::StaticIpProvisioner => 21,

            Plugin::Anaconda => 25,
            Plugin::Python => 25,

            Plugin::Rust => 26,
            Plugin::Go => 27,
            Plugin::Docker => 28,
            Plugin::Containerd => 29,
            Plugin::Runc => 30,
            Plugin::CniPlugins => 31,

            Plugin::AwsCfnHelper => 32,
            Plugin::Saml2Aws => 33,

            Plugin::AwsIamAuthenticator => 34,
            Plugin::EcrCredentialHelper => 35,
            Plugin::EcrCredentialProvider => 36,

            Plugin::Kubelet => 37,
            Plugin::Kubectl => 38,
            Plugin::Helm => 50,
            Plugin::Terraform => 51,

            Plugin::SshKeyWithEmail => 68,

            Plugin::Ena => 100,

            Plugin::NvidiaDriver => 200,
            Plugin::NvidiaCudaToolkit => 201,
            Plugin::NvidiaContainerToolkit => 202,

            Plugin::AmdRadeonGpuDriver => 300,

            Plugin::ProtobufCompiler => 60000,
            Plugin::Cmake => 60001,
            Plugin::Gcc7 => 60002,

            Plugin::DevBark => 80000,
            Plugin::DevFaissGpu => 80001,

            Plugin::EksWorkerNodeAmiScratch => 99990,
            Plugin::EksWorkerNodeAmiReuse => 99991,

            Plugin::AmiInfo => u32::MAX - 2000,
            Plugin::ClusterInfo => u32::MAX - 1999,

            Plugin::PostInitScript => u32::MAX - 1000,

            Plugin::CleanupImagePackages => u32::MAX - 10,
            Plugin::CleanupImageTmpDir => u32::MAX - 9,
            Plugin::CleanupImageAwsCredentials => u32::MAX - 8,
            Plugin::CleanupImageSshKeys => u32::MAX - 5,

            Plugin::Unknown(_) => u32::MAX,
        }
    }

    /// Returns all the `&str` values of the enum members.
    pub fn values() -> &'static [&'static str] {
        &[
            "imds",                          //
            "provider-id",                   //
            "vercmp",                        //
            "setup-local-disks",             //
            "mount-bpf-fs",                  //
            "time-sync",                     //
            "system-limit-bump",             //
            "aws-cli",                       //
            "ssm-agent",                     //
            "cloudwatch-agent",              //
            "static-volume-provisioner",     //
            "static-ip-provisioner",         //
            "anaconda",                      //
            "python",                        //
            "rust",                          //
            "go",                            //
            "docker",                        //
            "containerd",                    //
            "runc",                          //
            "cni-plugins",                   //
            "protobuf-compiler",             //
            "aws-cfn-helper",                //
            "saml2aws",                      //
            "aws-iam-authenticator",         //
            "ecr-credential-helper",         //
            "ecr-credential-provider",       //
            "kubelet",                       //
            "kubectl",                       //
            "helm",                          //
            "terraform",                     //
            "ssh-key-with-email",            //
            "ena",                           //
            "nvidia-driver",                 //
            "nvidia-cuda-toolkit",           //
            "nvidia-container-toolkit",      //
            "amd-radeon-gpu-driver",         //
            "cmake",                         //
            "gcc7",                          //
            "dev-bark",                      //
            "dev-faiss-gpu",                 //
            "eks-worker-node-ami-scratch",   //
            "eks-worker-node-ami-reuse",     //
            "ami-info",                      //
            "cluster-info",                  //
            "post-init-script",              //
            "cleanup-image-packages",        //
            "cleanup-image-ssh-keys",        //
            "cleanup-image-aws-credentials", //
        ]
    }

    pub fn all() -> Vec<String> {
        vec![
            Plugin::Imds.as_str().to_string(),
            Plugin::ProviderId.as_str().to_string(),
            Plugin::Vercmp.as_str().to_string(),
            Plugin::SetupLocalDisks.as_str().to_string(),
            Plugin::MountBpfFs.as_str().to_string(),
            Plugin::TimeSync.as_str().to_string(),
            Plugin::SystemLimitBump.as_str().to_string(),
            Plugin::AwsCli.as_str().to_string(),
            Plugin::SsmAgent.as_str().to_string(),
            Plugin::CloudwatchAgent.as_str().to_string(),
            Plugin::StaticVolumeProvisioner.as_str().to_string(),
            Plugin::StaticIpProvisioner.as_str().to_string(),
            Plugin::Anaconda.as_str().to_string(),
            Plugin::Python.as_str().to_string(),
            Plugin::Rust.as_str().to_string(),
            Plugin::Go.as_str().to_string(),
            Plugin::Docker.as_str().to_string(),
            Plugin::Containerd.as_str().to_string(),
            Plugin::Runc.as_str().to_string(),
            Plugin::CniPlugins.as_str().to_string(),
            Plugin::AwsCfnHelper.as_str().to_string(),
            Plugin::Saml2Aws.as_str().to_string(),
            Plugin::AwsIamAuthenticator.as_str().to_string(),
            Plugin::EcrCredentialHelper.as_str().to_string(),
            Plugin::EcrCredentialProvider.as_str().to_string(),
            Plugin::Kubelet.as_str().to_string(),
            Plugin::Kubectl.as_str().to_string(),
            Plugin::Helm.as_str().to_string(),
            Plugin::Terraform.as_str().to_string(),
            Plugin::SshKeyWithEmail.as_str().to_string(),
            Plugin::Ena.as_str().to_string(),
            Plugin::NvidiaDriver.as_str().to_string(),
            Plugin::NvidiaCudaToolkit.as_str().to_string(),
            Plugin::NvidiaContainerToolkit.as_str().to_string(),
            Plugin::AmdRadeonGpuDriver.as_str().to_string(),
            Plugin::ProtobufCompiler.as_str().to_string(),
            Plugin::Cmake.as_str().to_string(),
            Plugin::Gcc7.as_str().to_string(),
            Plugin::DevBark.as_str().to_string(),
            Plugin::DevFaissGpu.as_str().to_string(),
            Plugin::EksWorkerNodeAmiScratch.as_str().to_string(),
            Plugin::EksWorkerNodeAmiReuse.as_str().to_string(),
            Plugin::AmiInfo.as_str().to_string(),
            Plugin::ClusterInfo.as_str().to_string(),
            Plugin::PostInitScript.as_str().to_string(),
            Plugin::CleanupImagePackages.as_str().to_string(),
            Plugin::CleanupImageTmpDir.as_str().to_string(),
            Plugin::CleanupImageAwsCredentials.as_str().to_string(),
            Plugin::CleanupImageSshKeys.as_str().to_string(),
        ]
    }

    pub fn default() -> Vec<String> {
        vec![
            Plugin::Imds.as_str().to_string(),
            Plugin::ProviderId.as_str().to_string(),
            Plugin::Vercmp.as_str().to_string(),
            Plugin::SetupLocalDisks.as_str().to_string(),
            Plugin::MountBpfFs.as_str().to_string(),
            Plugin::TimeSync.as_str().to_string(),
            Plugin::SystemLimitBump.as_str().to_string(),
            Plugin::SsmAgent.as_str().to_string(),
            Plugin::CloudwatchAgent.as_str().to_string(),
            Plugin::Anaconda.as_str().to_string(),
            Plugin::AwsCfnHelper.as_str().to_string(),
        ]
    }
}

impl AsRef<str> for Plugin {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// To be printed/echoed before clean up process (e.g., before cleaning up SSH keys).
pub const INIT_SCRIPT_COMPLETE_MSG: &str = "INIT SCRIPT COMPLETE";

pub fn create(
    arch_type: ec2::ArchType,
    os_type: ec2::OsType,
    plugins_str: Vec<String>,
    require_static_ip_provisioner: bool,
    s3_bucket: &str,
    id: &str,
    region: &str,
    volume_type: &str,
    volume_size: u32,
    volume_iops: u32,
    volume_throughput: u32,
    ssh_key_email: Option<String>,
    aws_secret_key_id: Option<String>,
    aws_secret_access_key: Option<String>,
    post_init_script: Option<String>,
) -> io::Result<(Vec<Plugin>, String)> {
    let mut plugins_set = HashSet::new();
    for p in plugins_str.iter() {
        let plugin = Plugin::from_str(p).map_err(|e| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("failed to convert '{p}' to plugin {}", e),
            )
        })?;
        plugins_set.insert(plugin);
    }

    if plugins_set.contains(&Plugin::StaticVolumeProvisioner) {
        if require_static_ip_provisioner {
            plugins_set.insert(Plugin::StaticIpProvisioner);
        }
    }
    // pick either anaconda or python
    if plugins_set.contains(&Plugin::Anaconda) {
        if plugins_set.contains(&Plugin::Python) {
            log::info!("anaconda specifies thus overriding python plugin");
            plugins_set.remove(&Plugin::Python);
        }
    } else if plugins_set.contains(&Plugin::Python) {
        log::info!("only python plugin, without anaconda");
    }
    if arch_type.is_nvidia() {
        plugins_set.insert(Plugin::NvidiaDriver);
    }

    if plugins_set.contains(&Plugin::AwsCfnHelper) {
        if !plugins_set.contains(&Plugin::Anaconda) && !plugins_set.contains(&Plugin::Python) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' requires pip/python install plugin",
                    Plugin::AwsCfnHelper.as_str()
                ),
            ));
        }
    }
    if plugins_set.contains(&Plugin::EcrCredentialProvider) && !plugins_set.contains(&Plugin::Go) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "'{}' requires '{}' plugin",
                Plugin::EcrCredentialProvider.as_str(),
                Plugin::Go.as_str(),
            ),
        ));
    }
    if plugins_set.contains(&Plugin::TimeSync) && !plugins_set.contains(&Plugin::Imds) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "'{}' requires '{}' plugin",
                Plugin::TimeSync.as_str(),
                Plugin::Imds.as_str(),
            ),
        ));
    }
    if plugins_set.contains(&Plugin::Ena) && !plugins_set.contains(&Plugin::Imds) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "'{}' requires '{}' plugin",
                Plugin::Ena.as_str(),
                Plugin::Imds.as_str(),
            ),
        ));
    }
    if plugins_set.contains(&Plugin::NvidiaCudaToolkit) {
        if !arch_type.is_nvidia() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' but arch type is '{}' (not nvidia)",
                    Plugin::NvidiaCudaToolkit.as_str(),
                    arch_type.as_str()
                ),
            ));
        }
        if !plugins_set.contains(&Plugin::NvidiaDriver) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' but no '{}'",
                    Plugin::NvidiaCudaToolkit.as_str(),
                    Plugin::NvidiaDriver.as_str()
                ),
            ));
        }
    }
    if plugins_set.contains(&Plugin::NvidiaCudaToolkit) {
        if !arch_type.is_nvidia() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' but arch type is '{}' (not nvidia)",
                    Plugin::NvidiaCudaToolkit.as_str(),
                    arch_type.as_str()
                ),
            ));
        }
        if !plugins_set.contains(&Plugin::NvidiaDriver) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' but no '{}'",
                    Plugin::NvidiaContainerToolkit.as_str(),
                    Plugin::NvidiaDriver.as_str()
                ),
            ));
        }
    }

    if plugins_set.contains(&Plugin::DevBark) {
        if !plugins_set.contains(&Plugin::StaticVolumeProvisioner) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' requires '{}'",
                    Plugin::DevBark.as_str(),
                    Plugin::StaticVolumeProvisioner.as_str()
                ),
            ));
        }
    }
    if plugins_set.contains(&Plugin::DevFaissGpu) {
        if !plugins_set.contains(&Plugin::StaticVolumeProvisioner) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "specified '{}' requires '{}'",
                    Plugin::DevFaissGpu.as_str(),
                    Plugin::StaticVolumeProvisioner.as_str()
                ),
            ));
        }
    }

    if plugins_set.contains(&Plugin::EksWorkerNodeAmiScratch)
        && plugins_set.contains(&Plugin::EksWorkerNodeAmiReuse)
    {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "'{}' conflicts with '{}'",
                Plugin::EksWorkerNodeAmiScratch.as_str(),
                Plugin::EksWorkerNodeAmiReuse.as_str()
            ),
        ));
    }

    if post_init_script.is_some() {
        if !plugins_set.contains(&Plugin::PostInitScript) {
            log::warn!(
                "'{}' but no post init script specified -- adding to set",
                Plugin::PostInitScript.as_str()
            );
        }
        plugins_set.insert(Plugin::PostInitScript);
    }

    let mut plugins = Vec::new();
    for p in plugins_set.iter() {
        plugins.push(p.clone());
    }
    plugins.sort();

    // TODO: make this configurable?
    let provisioner_initial_wait_random_seconds = 10;

    let mut contents = scripts::start(os_type.clone())?;
    let mut updated_bash_profile = false;
    for p in plugins.iter() {
        match p {
            Plugin::Imds => {
                let d = scripts::imds(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::ProviderId => {
                let d = scripts::provider_id(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Vercmp => {
                let d = scripts::vercmp(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::SetupLocalDisks => {
                let d = scripts::setup_local_disks(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::MountBpfFs => {
                let d = scripts::mount_bpf_fs(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::TimeSync => {
                let d = scripts::time_sync(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::SystemLimitBump => {
                let d = scripts::system_limit_bump(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);

                if !updated_bash_profile {
                    updated_bash_profile = true;

                    let d = scripts::update_bash_profile(
                        os_type.clone(),
                        plugins_set.contains(&Plugin::Anaconda),
                        plugins_set.contains(&Plugin::Python),
                        plugins_set.contains(&Plugin::Rust),
                        plugins_set.contains(&Plugin::NvidiaCudaToolkit),
                        plugins_set.contains(&Plugin::Go),
                        plugins_set.contains(&Plugin::Kubectl),
                        plugins_set.contains(&Plugin::Helm),
                        plugins_set.contains(&Plugin::StaticVolumeProvisioner),
                    )?;
                    contents.push_str(
                        "###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n",
                    );
                    contents.push_str(&d);
                }
            }
            Plugin::AwsCli => {
                let d = scripts::aws_cli(arch_type.clone(), os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::SsmAgent => {
                let d = scripts::ssm_agent(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::CloudwatchAgent => {
                let d = scripts::cloudwatch_agent(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::StaticVolumeProvisioner => {
                // https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-launchtemplate-blockdevicemapping-ebs.html#cfn-ec2-launchtemplate-blockdevicemapping-ebs-volumetype
                // https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-launchtemplate-blockdevicemapping-ebs.html#cfn-ec2-launchtemplate-blockdevicemapping-ebs-volumesize
                // https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-launchtemplate-blockdevicemapping-ebs.html#cfn-ec2-launchtemplate-blockdevicemapping-ebs-iops
                //
                // only for gp3
                // https://aws.amazon.com/ebs/volume-types/
                // https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-launchtemplate-blockdevicemapping-ebs.html#cfn-ec2-launchtemplate-blockdevicemapping-ebs-throughput
                // "1000" does not work -- "InvalidParameterValue - Throughput (MiBps) to iops ratio of 0.333333 is too high; maximum is 0.250000 MiBps per iops."
                let d = scripts::static_volume_provisioner(
                    os_type.clone(),
                    id,
                    region,
                    volume_type,
                    volume_size,
                    volume_iops,
                    volume_throughput,
                    "/dev/xvdb",
                    provisioner_initial_wait_random_seconds,
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);

                if !updated_bash_profile {
                    updated_bash_profile = true;

                    let d = scripts::update_bash_profile(
                        os_type.clone(),
                        plugins_set.contains(&Plugin::Anaconda),
                        plugins_set.contains(&Plugin::Python),
                        plugins_set.contains(&Plugin::Rust),
                        plugins_set.contains(&Plugin::NvidiaCudaToolkit),
                        plugins_set.contains(&Plugin::Go),
                        plugins_set.contains(&Plugin::Kubectl),
                        plugins_set.contains(&Plugin::Helm),
                        plugins_set.contains(&Plugin::StaticVolumeProvisioner),
                    )?;
                    contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                    contents.push_str(&d);
                }
            }
            Plugin::StaticIpProvisioner => {
                let d = scripts::static_ip_provisioner(
                    os_type.clone(),
                    id,
                    region,
                    provisioner_initial_wait_random_seconds,
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::Anaconda => {
                let d = scripts::anaconda(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Python => {
                let d = scripts::python(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::Rust => {
                let d = scripts::rust(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Go => {
                let d = scripts::go(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::Docker => {
                let d = scripts::docker(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Containerd => {
                let d = scripts::containerd(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Runc => {
                let d = scripts::runc(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::CniPlugins => {
                let d = scripts::cni_plugins(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::AwsCfnHelper => {
                let d = scripts::aws_cfn_helper(
                    os_type.clone(),
                    if plugins_set.contains(&Plugin::Anaconda) {
                        "/home/ubuntu/anaconda3/bin"
                    } else {
                        ""
                    },
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Saml2Aws => {
                let d = scripts::saml2aws(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::AwsIamAuthenticator => {
                let d = scripts::aws_iam_authenticator(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::EcrCredentialHelper => {
                let d = scripts::ecr_credential_helper(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::EcrCredentialProvider => {
                let d = scripts::ecr_credential_provider(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::Kubelet => {
                let d = scripts::kubelet(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Kubectl => {
                let d = scripts::kubectl(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Helm => {
                let d = scripts::helm(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Terraform => {
                let d = scripts::terraform(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::SshKeyWithEmail => {
                if ssh_key_email.is_none() {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "plugin {} specified but empty email",
                            Plugin::SshKeyWithEmail.as_str()
                        ),
                    ));
                }
                let d = scripts::ssh_key_with_email(
                    os_type.clone(),
                    ssh_key_email.clone().unwrap().as_str(),
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::Ena => {
                let d = scripts::ena(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::NvidiaDriver => {
                let d = scripts::nvidia_driver(arch_type.clone(), os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::NvidiaCudaToolkit => {
                let d = scripts::nvidia_cuda_toolkit(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::NvidiaContainerToolkit => {
                let d = scripts::nvidia_container_toolkit(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::AmdRadeonGpuDriver => {
                let d = scripts::amd_radeon_gpu_driver(arch_type.clone(), os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::ProtobufCompiler => {
                let d = scripts::protobuf_compiler(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Cmake => {
                let d = scripts::cmake(
                    os_type.clone(),
                    if plugins_set.contains(&Plugin::Anaconda) {
                        "/home/ubuntu/anaconda3/bin"
                    } else {
                        ""
                    },
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::Gcc7 => {
                let d = scripts::gcc7(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::DevBark => {
                let d = scripts::dev_bark(
                    os_type.clone(),
                    if plugins_set.contains(&Plugin::Anaconda) {
                        "/home/ubuntu/anaconda3/bin"
                    } else {
                        ""
                    },
                    plugins_set.contains(&Plugin::StaticVolumeProvisioner),
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::DevFaissGpu => {
                let d = scripts::dev_faiss_gpu(
                    os_type.clone(),
                    plugins_set.contains(&Plugin::StaticVolumeProvisioner),
                )?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::EksWorkerNodeAmiScratch => {
                let d = scripts::eks_worker_node_ami_scratch(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }
            Plugin::EksWorkerNodeAmiReuse => {
                let d = scripts::eks_worker_node_ami_reuse(os_type.clone())?;
                contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
                contents.push_str(&d);
            }

            Plugin::AmiInfo
            | Plugin::ClusterInfo
            | Plugin::PostInitScript
            | Plugin::CleanupImagePackages
            | Plugin::CleanupImageTmpDir
            | Plugin::CleanupImageAwsCredentials
            | Plugin::CleanupImageSshKeys => {
                log::info!(
                    "skipping {}, saving it to write after bash profile at the end",
                    p.as_str()
                )
            }

            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("unknown plugin {}", p.as_str()),
                ))
            }
        }
    }

    if !updated_bash_profile {
        let d = scripts::update_bash_profile(
            os_type.clone(),
            plugins_set.contains(&Plugin::Anaconda),
            plugins_set.contains(&Plugin::Python),
            plugins_set.contains(&Plugin::Rust),
            plugins_set.contains(&Plugin::NvidiaCudaToolkit),
            plugins_set.contains(&Plugin::Go),
            plugins_set.contains(&Plugin::Kubectl),
            plugins_set.contains(&Plugin::Helm),
            plugins_set.contains(&Plugin::StaticVolumeProvisioner),
        )?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }

    if let Some(secret_key_id) = &aws_secret_key_id {
        let access_key = aws_secret_access_key.clone().unwrap();
        let d = scripts::aws_key(os_type.clone(), region, secret_key_id.as_str(), &access_key)?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }

    if plugins_set.contains(&Plugin::AmiInfo) {
        let d = scripts::ami_info(os_type.clone())?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }
    if plugins_set.contains(&Plugin::ClusterInfo) {
        let d = scripts::cluster_info(
            s3_bucket,
            id,
            plugins_set.contains(&Plugin::StaticVolumeProvisioner),
        );

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }

    if plugins_set.contains(&Plugin::PostInitScript) {
        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str("###########################\n# USER-DEFINED POST INIT SCRIPT\n");
        contents.push_str(&post_init_script.unwrap());
    }

    if plugins_set.contains(&Plugin::CleanupImagePackages) {
        let d = scripts::cleanup_image_packages(os_type.clone())?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }
    if plugins_set.contains(&Plugin::CleanupImageTmpDir) {
        let d = scripts::cleanup_image_tmp_dir(os_type.clone())?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }

    if plugins_set.contains(&Plugin::CleanupImageAwsCredentials) {
        let d = scripts::cleanup_image_aws_credentials(os_type.clone())?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }

    // do before the SSH key clean-ups
    contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
    contents
        .push_str(format!("###########################\n# {INIT_SCRIPT_COMPLETE_MSG}\n").as_str());
    contents.push_str(format!("echo \"{INIT_SCRIPT_COMPLETE_MSG}\"\n\n").as_str());

    if plugins_set.contains(&Plugin::CleanupImageSshKeys) {
        let d = scripts::cleanup_image_ssh_keys(os_type.clone())?;

        contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
        contents.push_str(&d);
    }

    contents.push_str("###########################\nset +x\necho \"\"\necho \"\"\necho \"\"\necho \"\"\necho \"\"\nset -x\n\n\n\n\n");
    Ok((plugins, contents))
}

pub fn to_strings(plugins: Vec<Plugin>) -> Vec<String> {
    let mut ss = Vec::new();
    for s in plugins.iter() {
        ss.push(s.as_str().to_string());
    }
    ss
}

impl Ord for Plugin {
    fn cmp(&self, plugin: &Plugin) -> std::cmp::Ordering {
        self.rank().cmp(&(plugin.rank()))
    }
}

impl PartialOrd for Plugin {
    fn partial_cmp(&self, plugin: &Plugin) -> Option<std::cmp::Ordering> {
        Some(self.cmp(plugin))
    }
}

impl PartialEq for Plugin {
    fn eq(&self, plugin: &Plugin) -> bool {
        self.cmp(plugin) == std::cmp::Ordering::Equal
    }
}

/// RUST_LOG=debug cargo test --package aws-manager --lib -- ec2::plugins::test_sort --exact --show-output
#[test]
fn test_sort() {
    let expected: Vec<Plugin> = vec![
        Plugin::Imds,
        Plugin::ProviderId,
        Plugin::Vercmp,
        Plugin::SetupLocalDisks,
        Plugin::MountBpfFs,
        Plugin::TimeSync,
        Plugin::SystemLimitBump,
        Plugin::AwsCli,
        Plugin::SsmAgent,
        Plugin::CloudwatchAgent,
        Plugin::Anaconda,
        Plugin::Go,
        Plugin::Docker,
        Plugin::Containerd,
        Plugin::Runc,
        Plugin::Ena,
        Plugin::NvidiaDriver,
        Plugin::DevBark,
        Plugin::EksWorkerNodeAmiReuse,
        Plugin::AmiInfo,
        Plugin::PostInitScript,
        Plugin::CleanupImagePackages,
        Plugin::CleanupImageTmpDir,
        Plugin::CleanupImageAwsCredentials,
        Plugin::CleanupImageSshKeys,
    ];

    let mut unsorted: Vec<Plugin> = vec![
        Plugin::NvidiaDriver,
        Plugin::EksWorkerNodeAmiReuse,
        Plugin::CloudwatchAgent,
        Plugin::CleanupImageSshKeys,
        Plugin::Runc,
        Plugin::CleanupImagePackages,
        Plugin::CleanupImageTmpDir,
        Plugin::PostInitScript,
        Plugin::CleanupImageAwsCredentials,
        Plugin::SsmAgent,
        Plugin::MountBpfFs,
        Plugin::Imds,
        Plugin::SystemLimitBump,
        Plugin::Containerd,
        Plugin::Vercmp,
        Plugin::DevBark,
        Plugin::ProviderId,
        Plugin::TimeSync,
        Plugin::Docker,
        Plugin::SetupLocalDisks,
        Plugin::Anaconda,
        Plugin::AmiInfo,
        Plugin::Go,
        Plugin::AwsCli,
        Plugin::Ena,
    ];
    unsorted.sort();

    assert_eq!(expected, unsorted);
    assert_eq!(unsorted[0], Plugin::Imds);
}
