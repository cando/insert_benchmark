#!/bin/bash
export $(cat .$1.env | xargs) && cargo run --release
