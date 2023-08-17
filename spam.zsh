#!/bin/zsh

for i in {101..201}; do
    data="username=user$i&password=admin&email=user$i@gmail.com"
    curl -X POST -b "access_token=testing.jorkridesher.testing" -d "$data" -H "Content-Type: application/x-www-form-urlencoded" http://localhost:33141/auth/account
done
