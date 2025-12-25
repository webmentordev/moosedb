#!/bin/bash

cd ui && npx nuxi generate && cd .. && cargo build --release