#!/bin/bash

echo "Building XTerminal Pro for browser..."

# Node modules should already be installed
npm run build

echo "Build complete!"
echo "Open dist/index.html in your browser to run XTerminal Pro"