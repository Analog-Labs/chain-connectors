#!/bin/sh

docker stop $(docker ps -a -q)
docker container rm $(docker container ls -a -q)
docker volume rm $(docker volume ls)
docker network rm $(docker network ls -q)
