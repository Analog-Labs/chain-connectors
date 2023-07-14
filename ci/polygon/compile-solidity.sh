#!/usr/bin/env sh
set -e

# This script builds a static binary of the specified version of Solidity
# Reference:
# https://github.com/ethereum/solc-bin/blob/c955cc37b2f132dd40ea6cfd89cb338a7ecaac2c/.github/workflows/random-macosx-build.yml#L180-L265

SOLIDITY_VERSION="$1"

git clone --recursive https://github.com/ethereum/solidity.git --branch "$SOLIDITY_VERSION" --depth 50 solidity
cd solidity

LAST_COMMIT_HASH=$(git rev-parse --short=8 HEAD)
FULL_BUILD_VERSION="v$SOLIDITY_VERSION+commit.$LAST_COMMIT_HASH"

# In 0.3.6 boostTest.cpp has its own main() function which leads to "duplicate symbol '_main'" linker error
# despite BOOST_TEST_NO_MAIN being defined. 0.4.0 does not have this problem so here we just backport that change.
if npx semver --range '= 0.3.6' "$SOLIDITY_VERSION"; then
    # NOTE: Only the first commit should be necessary but one of its changes seems to
    # have been mistakenly included in the other and it won't compile on its own.
    git cherry-pick 1bc0320811ef2b213bda0629b702bffae5e2f925 # [PR #837] Cleanup of test suite init.
    git cherry-pick 53f68a155f071194fd779352d5997c03a6c387ed # [PR #837] Exponential sleep increase on mining failure.
fi

# Static linking with jsoncpp was introduced in 0.4.5
if npx semver --range '>= 0.4.2 <= 0.4.4' "$SOLIDITY_VERSION"; then
    # The change can be backported from 0.4.5 and applies cleanly between 0.4.2 and 0.4.4.
    git cherry-pick 4bde0a2d36297c4b3fa17c7dac2bb1681e1e7f75 # [#1252] Build jsoncpp from source using jsoncpp.cmake script
elif npx semver --range '>= 0.3.6 <= 0.4.1' "$SOLIDITY_VERSION"; then
    # The commit won't apply cleanly before 0.4.2 due to conflicting changes in install_deps.sh and .travis.yml.
    # Those files don't affect our build so just reset the unmerged files.
    # NOTE: core.editor setting needs to be overridden for prevent git from asking us to edit the commit message.
    git cherry-pick 4bde0a2d36297c4b3fa17c7dac2bb1681e1e7f75 || true # [PR #1252] Build jsoncpp from source using jsoncpp.cmake script
    git reset -- scripts/install_deps.sh .travis.yml
    git checkout scripts/install_deps.sh .travis.yml
    git -c core.editor=/usr/bin/true cherry-pick --continue
fi

# Between 0.3.6 and 0.4.16 deps/ was a submodule and contained parts of the cmake configuration
if npx semver --range '>= 0.3.6 <= 0.4.16' "$SOLIDITY_VERSION"; then
    git submodule init
    git submodule update
fi

# Remove the -Werror flag. Unfortunately versions older than 0.6.1 do not compile without warnings
# when using a recent clang version on MacOS 10.15.
if npx semver --range '>= 0.3.6 <= 0.6.0' "$SOLIDITY_VERSION"; then
    sed -i.bak '/^[[:blank:]]*add_compile_options(-Werror)[[:blank:]]*$/d' cmake/EthCompilerSettings.cmake
fi

# Pre-0.5.0 versions were using wrong boost test header, resulting in the 'duplicate symbol' linker error in static builds.
# See https://github.com/ethereum/solidity/pull/4572
if npx semver --range '< 0.5.0' "$SOLIDITY_VERSION"; then
    sed -i.bak 's|^[[:blank:]]*#include <boost/test/included/unit_test\.hpp>[[:blank:]]*$|#include <boost/test/unit_test.hpp>|g' test/boostTest.cpp
fi

# Uncommitted changes result in .mod (or -mod in 0.4.0) being included in version string which affects
# bytecode produced by the compiler. Committing them would change the commit hash so that's not
# an option. We have to disable this if we want the output to match a release build.
if npx semver --range '>= 0.4.0 <= 0.6.0' "$SOLIDITY_VERSION"; then
    sed -i.bak '/^[[:blank:]]*set(SOL_COMMIT_HASH \"\${SOL_COMMIT_HASH}[-.]mod\")[[:blank:]]*$/d' cmake/scripts/buildinfo.cmake
fi

# Starting with 0.4.16 there's no STATIC_LINKING option but there's SOLC_LINK_STATIC instead.
if npx semver --range '>= 0.4.16' "$SOLIDITY_VERSION"; then
    static_linking_option="-DSOLC_LINK_STATIC=1"
else
    static_linking_option="-DSTATIC_LINKING=1"
fi

mkdir -p build/
echo -n > prerelease.txt
cd build/

cmake .. "${static_linking_option}" -DCMAKE_BUILD_TYPE=Release -G "Unix Makefiles"
make -j 3 solc

echo "$FULL_BUILD_VERSION"
