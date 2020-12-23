import sys
import os.path
import subprocess

VTAS_BINARY = "./target/debug/vtas"


def run_test():
    print("Running vtas tests...")
    subprocess.run([VTAS_BINARY, "test"])


def tests_command():
    if os.path.isfile(VTAS_BINARY):
        run_test()
    else:
        subprocess.run(["cargo build"])
        run_test()


commands = {
    "test": tests_command
}

if len(sys.argv) >= 1:
    print("Command required")
else:
    commands.get(sys.argv[1])()
