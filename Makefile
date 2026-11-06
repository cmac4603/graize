.PHONY: run-controller run-frontend install-helm

run-controller:
	cargo run -p controller

run-frontend:
	topcoat dev -p frontend

install-helm:
	helm upgrade --install graize .helm-charts --namespace graize --create-namespace
