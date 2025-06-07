#!/bin/bash

read -p "Enter event name: " EVENT_NAME
read -p "Enter new status: " STATUS

curl -X POST "http://localhost:8000/api/v3/update/$EVENT_NAME/$STATUS"
