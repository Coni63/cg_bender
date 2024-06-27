import datetime
import subprocess
import time
from pathlib import Path


test_files = [f"tests/{i}.txt" for i in range(1, 31)]

ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")

folder = Path(f"results/{ts}")
folder.mkdir(parents=True, exist_ok=True)

total_length = 0
total_win = 0
for i, test_file in enumerate(test_files):
    
    start_time = time.time()

    result = subprocess.run(['python', 'main.py'], stdin=open(test_file, 'r'), capture_output=True, text=True)
    output_string = result.stdout.strip()
    
    end_time = time.time()

    solution_found = len(output_string) > 0
    time_limit_exceeded = end_time - start_time >= 1

    with open(folder / f"{i+1}.txt", 'w') as f:
        f.write(output_string)

    if solution_found and not time_limit_exceeded:
        total_win += 1
        total_length += len(output_string)
        print(f"Test {i+1}: 'OK' ({end_time - start_time:0.3f})")
    elif solution_found and time_limit_exceeded:
        total_length += len(output_string)
        print(f"Test {i+1}: 'TIMEOUT' ({end_time - start_time:0.3f})")
    else:
        print(f"Test {i+1}: 'FAIL'")

# Print the total length of all output strings
print(f"\n{total_win}/30 tests passed - Total length: {total_length} characters")