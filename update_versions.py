#!/usr/bin/python3
import subprocess

def increment_version(version: str) -> str:
    version = version.split('.')
    version[-1] = str(int(version[-1]) + 1)
    return ".".join(version)


def update_cargo(util: str):
    with open(f"{util}/Cargo.toml", 'a+') as stream:
        stream.seek(0)
        lines = stream.read().split("\n")

        i = -1
        for line in lines:
            i += 1
            if not line.startswith("version"): continue
            version_no = increment_version(line.split("=")[-1].strip(" \""))
            break

        lines[i] = f"version = \"{version_no}\""
        lines = "\n".join(lines) 

        stream.seek(0)
        stream.write(lines)

        subprocess.run(["git", "add", f"{util}/Cargo.toml"], capture_output=True)


def update_charon(util: str):
    with open(f"{util}/{util}.charon", 'a+') as stream:
        stream.seek(0)
        lines = stream.read().split("\n")

        # Find line to edit
        start = lines[0].find("version")
        end = lines[0][start:].find(",") + start
        line = lines[0][start:end]
        v1 = line.split("=")[-1].strip(" \"")
        v2 = increment_version(v1)

        # Update line.
        lines[0] = lines[0].replace(f"version = \"{v1}\"", f"version = \"{v2}\"")
        lines = "\n".join(lines) 

        # Write to file.
        stream.seek(0)
        stream.write(lines)

        # Add updated file to commit.
        subprocess.run(["git", "add", f"{util}/{util}.charon"], capture_output=True)


def main():
    # Read .charon/Cargo.toml.
    # Find version.
    # Increment version.
    # Update files.

    # Get utils that were modified.
    output = str(subprocess.run(["git", "diff", "--cached", "--name-only"], capture_output=True))
    
    if output.find("cocytus") != -1:
        update_cargo("cocytus")
        update_charon("cocytus")

    if output.find("styx") != -1:
        update_cargo("styx")
        update_charon("styx")

    if output.find("lethe") != -1:
        update_cargo("lethe")
        update_charon("lethe")

main()

