#!/bin/bash

echo "Building Termlane for browser..."

# Node modules should already be installed
npm run build

echo "Build complete!"
echo "Open dist/index.html in your browser to run Termlane"
