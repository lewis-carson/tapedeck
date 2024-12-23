import os
import json
import argparse

def get_files(path):
    return [os.path.join(path, f) for f in os.listdir(path)]

def line_generator(filepath):
    with open(filepath, 'r') as f:
        for line in f:
            if line.strip():
                yield json.loads(line)
    while True:
        yield None

def main():
    parser = argparse.ArgumentParser(description='Process files from specified directory')
    parser.add_argument('path', help='Path to the data directory')
    args = parser.parse_args()

    files = get_files(args.path)
    generators = [line_generator(f) for f in files]
    
    # Initialize data from all generators
    line_data = [next(gen) for gen in generators]

    while any(data is not None for data in line_data):
        times = [line['receive_time'] if line else float('inf') for line in line_data]
        smallest_time_index = times.index(min(times))

        print(line_data[smallest_time_index])

        # Get next line from the generator that had the smallest time
        line_data[smallest_time_index] = next(generators[smallest_time_index])

    
if __name__ == "__main__":
    main()


