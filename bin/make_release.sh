#!/bin/bash

# IBM Confidential
# PID 5737-N85, 5900-AG5
# Copyright IBM Corp. 2025

# Automates the steps described in README.md.
# Important: once it pushes the tag and the version file, the CI starts the
# release process.
# To stop the release process, interrupt the IBM Cloud pipeline under
# toolchain "OpenTelemetry cpp and rust based tracers" / `ci-pipeline` / trigger "JVM Profiler CI Pipeline on main runs"

set -e

SCRIPT_NAME="make_release.sh"
USAGE="\

Automation of steps to create a new release.

USAGE
  ${SCRIPT_NAME} [-d] -v <version>
  ${SCRIPT_NAME} [help|--help|-h]

OPTIONS:
  -v                Version expressed as X.Y.Z
  -d                Run in dryrun mode: Only display the expected commands.
  --help|-h    Display this help and exit.

Example:
 $0 -v 1.6.2 -d
"

# RELEASE_BRANCH="main"
ERROR=1
SUCCESS=0
SUCCESS_MSG="*** EXECUTION SUCCESSFUL ***"
VERSION_FILE="VERSION"

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

echo "Changing version to \"${VERSION}\" in the version file \"$VERSION_FILE\""
run_cmd "sed -i \"s/.*/$VERSION/\" \"$VERSION_FILE\""

echo "Committing the version file"
run_cmd "git add \"$VERSION_FILE\""
run_cmd "git commit -m \"Release ${VERSION}\""

echo "Creating tag with version \"${VERSION}\""
run_cmd "git tag -a \"${VERSION}\" -m \"Release ${VERSION}\""
echo "Pushing the tag"
run_cmd "git push origin \"${VERSION}\""

echo "Pushing the version file"
run_cmd "git push"

echo "${SUCCESS_MSG}"
exit "$SUCCESS"
