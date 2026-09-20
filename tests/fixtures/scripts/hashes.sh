#!/bin/bash

for file in ./objects/*.obj; do
	git hash-object "$file" > "./objects/$(basename "$file" ".obj").hash"
done
