install:
	docker-compose build
	cp ./rusty-slackbot.service /etc/systemd/system/rusty-slackbot.service
run:
	systemctl start rusty-slackbot.service


