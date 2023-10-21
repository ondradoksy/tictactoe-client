#!/bin/bash
cd "${0%/*}"
export NODE_OPTIONS=--openssl-legacy-provider
npm run start
