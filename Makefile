.PHONY: run-controller run-ui install-helm

run-controller:
	cargo run -p controller

run-ui:
	topcoat dev -p ui

install-helm:
	helm upgrade --install graize .helm-charts --namespace graize --create-namespace
