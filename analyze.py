import sys
from pprint import pprint


def parse_logs(logs_path):
    active_tasks = []
    all_tasks = []

    with open(logs_path, "r") as f:
        for line in f:
            if line.startswith("[T"):
                current_time = int(line.split("]")[0][3:])
                if "Start" in line:
                    task_name = line.split("Start ")[1][:-1]
                    active_tasks.append({"task_name": task_name, "start_time": current_time, "end_time": -1, "duration": -1, "msm": {}, "fft": {}})
                elif "Done" in line:
                    active_tasks[-1]["end_time"] = current_time
                    active_tasks[-1]["duration"] = current_time - active_tasks[-1]["start_time"]
                    all_tasks.append(active_tasks.pop())
            elif line.startswith("msm") or line.startswith("fft"):
                operation, size = line.split()
                if active_tasks:
                    if size in active_tasks[-1][operation]:
                        active_tasks[-1][operation][size] += 1
                    else:
                        active_tasks[-1][operation][size] = 1

    all_tasks = sorted(all_tasks, key=lambda task: task["duration"] * -1)
    for task in all_tasks:
        if task["task_name"] == "committing to advice columns":
            pprint(task)
            break
    # for task in all_tasks:
    #     if task["task_name"] == "committing to advice columns":
    #         pprint(task)
    #         break

if __name__ == "__main__":
    # logs = [
    #     {"name": "leader", "path": "./benchmark/out_leader.txt"},
    #     {"name": "worker_0", "path": "./benchmark/out_worker_0.txt"},
    #     {"name": "worker_1", "path": "./benchmark/out_worker_1.txt"},
    #     {"name": "single", "path": "./benchmark/out_single.txt"},
    # ]
    # for log in logs:
    #     print(log["name"])
    #     parse_logs(log["path"])
    parse_logs("./out.txt")
