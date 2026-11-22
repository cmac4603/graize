use anyhow::{Context, Result};
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client, api::ListParams};

use super::Deployer;

pub struct OpencodeWorkerDeployer {
    client: Client,
    namespace: String,
    image: String,
}

impl OpencodeWorkerDeployer {
    /// Build a deployer from kubeconfig or the controller's in-cluster service account.
    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .context("loading Kubernetes client configuration")?;
        let namespace = client.default_namespace().to_owned();
        let image = super::WORKER_IMAGE.to_string();

        Ok(Self {
            client,
            namespace,
            image,
        })
    }
}

impl Deployer for OpencodeWorkerDeployer {
    async fn deploy_worker<'w>(&self, worker_id: super::WorkerId<'w>) -> Result<Deployment> {
        let deployment = self.build_worker_deployment(worker_id, &self.image)?;
        let name = deployment
            .metadata
            .name
            .as_deref()
            .context("worker Deployment has no name")?;
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);

        deployments
            .create(&Default::default(), &deployment)
            .await
            .with_context(|| format!("creating worker Deployment {name}"))
    }

    async fn list_workers(&self) -> Result<Vec<Deployment>> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);
        deployments
            .list(&ListParams::default().labels(super::WORKER_LIST_SELECTOR))
            .await
            .context("listing worker Deployments")
            .map(|deployments| deployments.items)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn worker_deployment_has_one_worker_container_and_matching_selector() {
        let owd = OpencodeWorkerDeployer::new().await.unwrap();
        let deployment = owd
            .build_worker_deployment("task-123", "ghcr.io/cmac4603/graize/worker:test")
            .unwrap();
        let deployment: Value = serde_json::to_value(deployment).unwrap();

        assert_eq!(deployment["metadata"]["name"], "graize-worker-task-123");
        assert_eq!(deployment["spec"]["replicas"], 1);
        assert_eq!(
            deployment["metadata"]["labels"]["app.kubernetes.io/name"],
            "opencode-worker"
        );
        assert_eq!(
            deployment["spec"]["selector"]["matchLabels"],
            deployment["spec"]["template"]["metadata"]["labels"]
        );
        assert_eq!(
            deployment["spec"]["template"]["spec"]["containers"][0]["name"],
            "opencode-worker"
        );
        assert_eq!(
            deployment["spec"]["template"]["spec"]["containers"][0]["image"],
            "ghcr.io/cmac4603/graize/worker:test"
        );
        assert_eq!(
            deployment["spec"]["template"]["spec"]["containers"][0]["ports"][0]["containerPort"],
            4096
        );
    }

    #[tokio::test]
    async fn worker_id_must_be_a_dns_label() {
        let owd = OpencodeWorkerDeployer::new().await.unwrap();
        assert!(
            owd.build_worker_deployment("Task/123", "worker:latest")
                .is_err()
        );
    }
}
