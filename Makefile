install:
	docker build -t rusty-slackbot -f ./Dockerfile .
	cp ./rusty-slackbot.service /etc/systemd/system/rusty-slackbot.service
run:
	systemctl start rusty-slackbot.service


