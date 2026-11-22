/// Client to talk to the worker.
pub mod client;
/// Deploys workers to k8s.
pub mod deployer;
/// Opencode Server OpenAPI generated client & types.
mod generated;

use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use garde::{Valid, Validate};
use k8s_openapi::api::{
    apps::v1::{Deployment, DeploymentSpec},
    core::v1::{Container, ContainerPort, PodSpec, PodTemplateSpec},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use once_cell::sync::Lazy;

const DEFAULT_WORKER_IMAGE: &str = "ghcr.io/cmac4603/graize/worker:latest";
const WORKER_LABEL: &str = "app.kubernetes.io/name";
const WORKER_LABEL_VALUE: &str = "opencode-worker";
const WORKER_LIST_SELECTOR: &str = "app.kubernetes.io/name=opencode-worker";
const WORKER_NAME_PREFIX: &str = "graize-worker-";
const MAX_WORKER_ID_LENGTH: usize = 63 - WORKER_NAME_PREFIX.len();

static WORKER_IMAGE: Lazy<String> = Lazy::new(|| {
    std::env::var("WORKER_IMAGE").unwrap_or_else(|_| DEFAULT_WORKER_IMAGE.to_string())
});

#[derive(Validate)]
struct WorkerId<'w> {
    /// Used in metadata name & graize label name
    //https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#dns-label-names
    #[garde(length(min = 1, max = 63), pattern("^[a-z0-9]([-a-z0-9]*[a-z0-9])?$"))]
    uid: &'w str,
}

impl<'w> WorkerId<'w> {
    fn label_name(&self) -> String {
        self.uid.to_string()
    }

    fn metadata_name(&self) -> String {
        format!("{WORKER_NAME_PREFIX}{0}", self.uid)
    }

}

pub trait Deployer {
    /// Create a one-replica worker Deployment for a DNS-label-compatible worker ID.
    async fn deploy_worker(&self, worker_id: WorkerId) -> Result<Deployment>;

    /// List this controller's worker Deployments in its namespace.
    async fn list_workers(&self) -> Result<Vec<Deployment>>;

    /// Build the worker Kubernetes [Deployment]
    fn build_worker_deployment(&self, worker_id: WorkerId, image: &str) -> Result<Deployment> {
        ensure!(!image.trim().is_empty(), "worker image cannot be empty");

        let mut labels = BTreeMap::new();
        labels.insert(WORKER_LABEL.to_owned(), WORKER_LABEL_VALUE.to_owned());
        labels.insert(
            "app.kubernetes.io/instance".to_owned(),
            worker_id.label_name(),
        );

        Ok(Deployment {
            metadata: ObjectMeta {
                name: Some(worker_id.metadata_name()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "opencode-worker".to_owned(),
                            image: Some(image.to_owned()),
                            ports: Some(vec![ContainerPort {
                                name: Some("http".to_owned()),
                                container_port: 4096,
                                protocol: Some("TCP".to_owned()),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
