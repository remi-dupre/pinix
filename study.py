#!/usr/bin/env python3
import json
import time
from collections import defaultdict
from dataclasses import dataclass, field

from enum import Enum


class ActionType(Enum):
    UNKNOWN = 0
    COPY_PATH = 100
    FILE_TRANSFER = 101
    REALISE = 102
    COPY_PATHS = 103
    BUILDS = 104
    BUILD = 105
    OPTIMISE_STORE = 106
    VERIFY_PATHS = 107
    SUBSTITUTE = 108
    QUERY_PATH_INFO = 109
    POST_BUILD_HOOK = 110
    BUILD_WAITING = 111


@dataclass
class Progress:
    done: int = 0
    expected: int = 0
    running: int = 0
    failed: int = 0

    def __str__(self):
        return f"{self.done}/{self.expected} ({self.running})"


@dataclass
class BuildStep:
    step_id: int
    action_type: ActionType
    text: str = ""
    children: list[int] = field(default_factory=list)
    parent: int | None = None
    progress = Progress()

    def display(self, collection: dict[int, "BuildStep"], indent=0):
        indent_str = " " * indent * 2
        print(f"{indent_str}{self.action_type} {self.progress} {self.text}")

        for child in self.children:
            collection[child].display(collection, indent + 1)


def main():
    stderr = open("stderr.log").read()

    actions = {
        0: BuildStep(step_id=0, action_type=ActionType.BUILD),
    }

    for line in stderr.splitlines():
        try:
            _prefix, data = line.split(maxsplit=1)
            data = json.loads(data)
        except Exception:
            print("Bad line:", line, end="")
            continue

        match data["action"]:
            case "msg":
                print(data["msg"])
                continue
            case "start":
                step_id = data["id"]
                parent = data["parent"]
                actions[parent].children.append(step_id)

                actions[step_id] = BuildStep(
                    step_id=step_id,
                    action_type=ActionType(data["type"]),
                    parent=parent,
                    text=data["text"],
                )
            case "result":
                step_id = data["id"]

                if data["type"] == ActionType.BUILD.value:
                    actions[step_id].progress = Progress(*data["fields"])
                    # time.sleep(0.001)
                    # print()
    actions[0].display(actions)

    # if data["action"] == "msg":
    #     continue
    #
    # elem = per_id[data["id"]]
    # elem.update(data)
    # children[elem["parent"]].add(elem["id"])
    #
    # print()
    # todo = [0]
    # seen = set()
    #
    # while todo:
    #     curr = todo.pop()
    #
    #     if curr in seen:
    #         continue
    #
    #     seen.add(curr)
    #     todo += list(children[curr])
    #     elem = per_id[curr]
    #     print(curr, elem.get("action"), elem.get("fields"))


if __name__ == "__main__":
    main()
