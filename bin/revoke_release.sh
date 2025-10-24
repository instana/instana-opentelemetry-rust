#!/bin/bash

# IBM Confidential
# PID 5737-N85, 5900-AG5
# Copyright IBM Corp. 2025

# Revokes a release on main branch:
# 1. Check if HEAD commit message matches "Release $VERSION"
# 2. Creates a backup of main local branch
# 3. Delete the Release commit
# 4. Delete the release tag locally and remotely

set -euo pipefail

SCRIPT_NAME="revoke_release.sh"
USAGE="\

Automation of steps to revoke the latest release.
 1. Check if HEAD commit message matches \"Release <X.Y.Z>\"
 2. Creates a backup of main local branch
 3. Delete the Release commit
 4. Delete the release tag locally and remotely

USAGE
  ${SCRIPT_NAME} [-d] -v <version>
  ${SCRIPT_NAME} [help|--help|-h]

OPTIONS:
  -v                Version expressed as X.Y.Z
  -d                Run in dryrun mode: Only display the expected commands.
  --help|-h         Display this help and exit.

Example:
 $0 -v 1.1.13 -d
"

ERROR=1
SUCCESS=0
DRYRUN=0

if [ $# -lt 1 ]; then
  echo "${USAGE}"
  exit "$ERROR"
fi
OPTIONS=$(getopt -o "v:,d,h" -l "help" -- "$@")
eval set -- "${OPTIONS}"
while true ; do
  case $1 in
  -d)
    DRYRUN=1; shift;;
  -v)
    VERSION="$2"; shift 2;;
  -h | --help)
    echo "${USAGE}" ; exit 0;;
  --)
    shift; break;;
  esac
done

echo "The given version is: \"${VERSION}\""

# Validate the given version
if ! [[ $VERSION =~ ([0-9]+.){2}[0-9]+$ ]] ; then
  echo "Error: Version pattern not valid." >&2;
  exit $ERROR;
fi

# Source dryrun functions
SCRIPT_DIR=$(dirname "${BASH_SOURCE[0]}")
export DRYRUN
. "${SCRIPT_DIR}/utils.sh"

printf "\
#############################################################
### USING VERSION %-6s DRYRUN %-2s
#############################################################\n\n" "${VERSION}" "$DRYRUN"

echo -e "Exporting variable\n VERSION:\"${VERSION}\"\n"
export VERSION
pushd "${SCRIPT_DIR}/../"

# Step 1: Check if HEAD commit message matches "Release $VERSION"
echo -e "Checking if HEAD commit message matches \"Release ${VERSION}\"\n"
if [[ -z ${DRYRUN:-} || "${DRYRUN}" == "0" ]]; then
  commit_message=$(git log -1 --pretty=%s)
  if [[ "$commit_message" != "Release $VERSION" ]]; then
    echo "Error: HEAD commit does not contain 'Release $VERSION'. Exiting."
    exit $ERROR
  fi
else
  run_cmd "git log -1 --pretty=%s"
fi

# Step 2: Backup the branch (locally)
echo "Backup the current branch"
branch_name=$(git rev-parse --abbrev-ref HEAD)
backup_branch="${branch_name}_backup_$(date +%Y%m%d%H%M%S)"
run_cmd "git checkout -b \"$backup_branch\""

# Step 3: Rebase to remove the "Release $VERSION" commit
echo "Rebasing to remove the \"Release $VERSION\" commit"
run_cmd "git checkout \"$branch_name\""
run_cmd "git reset --soft HEAD^"  # Remove the HEAD commit but keep changes staged

# Step 4: Delete the release tag locally and remotely
echo "Deleting the release tag locally and remotely"
run_cmd "git tag -d $VERSION"
run_cmd "git push --delete origin $VERSION"

# Finally: Success message and reminder
echo "Success! IMPORTANT: Don't forget to do 'git push --force-with-lease origin $branch_name' to update the remote."
echo "NOTE: To recreate Release $VERSION, run bin/make_release.sh -v $VERSION."

exit "$SUCCESS"
