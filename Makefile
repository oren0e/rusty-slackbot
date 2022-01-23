install:
	docker-compose build
	cp ./rusty-slackbot.service /etc/systemd/system/rusty-slackbot.service
run:
	systemctl start rusty-slackbot.service

test:
	cargo test --verbose

lint:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings
