set -e
echo "args: $@"
echo xzcat
xzcat "$1" > tmp.tar
echo mkdir
mkdir out
echo cd
cd /out
echo tar
tar xvf ../tmp.tar
