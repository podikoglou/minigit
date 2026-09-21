#!/bin/bash

for file in ./objects/*.obj; do
  pigz -dz < "$file" | sha1sum | cut -d' ' -f1 > "./objects/$(basename "$file" ".obj").hash"
done
