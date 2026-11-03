.PHONY: run-controller run-ui

run-controller:
	cargo run -p controller

run-ui:
	topcoat dev -p ui
