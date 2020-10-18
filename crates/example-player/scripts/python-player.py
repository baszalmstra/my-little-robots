#!/usr/bin/python3
import sys
import json

def process(data: object):
    print(data)

def main():
    # Read json from stdin
    for line in sys.stdin:
        # Process json
        process(json.loads(line))
        break

if __name__ == '__main__':
    # Run the main
    main()
