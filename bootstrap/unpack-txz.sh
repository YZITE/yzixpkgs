set -e
xzcat "$1" > tmp.tar
mkdir out
cd /out
tar xf ../tmp.tar

shift
for i; do
  patch 
done

