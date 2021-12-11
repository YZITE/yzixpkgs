set -e
xzcat "$1" > tmp.tar
mkdir out
cd /out
tar xf ../tmp.tar

# yes, this is a hack.
if [ -d "$(echo *)" ]; then
  fld="$(echo *)"
  mkdir /newout
  mv -t/newout "$fld"/*
  rmdir "$fld"
  cd /
  rmdir /out
  mv -T /newout /out
  cd /out
fi

shift
ls -las
for i; do
  echo "apply patch $i"
  patch -p1 < "$i"
done
