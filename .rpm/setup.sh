#!/bin/bash -x

# name of the crate/package
name=$1
# version of the crate/package
version=$2
# commit to target (latest == master)
commit=$3
# path to the spec file on the pc
path_to_spec=$4
# repo link
repo=$5

LATEST="latest"

# Clone repo and cd into it
mkdir $name-$version && cd $name-$version && git clone --recurse-submodules $repo .

# Get latest commit hash if commit is set to latest
if [[ "$commit" == "$LATEST" ]]
then
    commit=$(git rev-parse HEAD)
fi

# Short commit, used for versioning
short_commit=${commit:0:6}

# Reset to specified commit
git reset --hard $commit

# Vendor dependencies and zip vendor
mkdir .vendor
cargo vendor > .vendor/config.toml
tar -pcJf $name-$version-vendor.tar.xz vendor && mv $name-$version-vendor.tar.xz ../$name-$version-vendor.tar.xz
# Back into parent directory
rm -rf vendor && cd ..

# Zip source
tar -pcJf $name-$version.tar.xz $name-$version
rm -rf $name-$version

# Get specfile
cp $path_to_spec $name.spec 2>/dev/null || :

# Make replacements to specfile
sed -i "/^%global ver / s/.*/%global ver $version/" $name.spec
sed -i "/^%global commit / s/.*/%global commit $commit/" $name.spec
current_date=$(date +'%Y%m%d.%H%M')
sed -i "/^%global date / s/.*/%global date $current_date/" $name.spec


ls -a
pwd

echo Done! $1 $2 $3 $4 $5