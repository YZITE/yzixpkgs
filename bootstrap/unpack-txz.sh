set -e
mkdir out
tar x -o -J -f "$1" -C /out

# yes, this is a hack.
if [ -d "$(echo /out/*)" ]; then
  fld="$(echo /out/*)"
  mkdir /newout
  mv -t/newout "$fld"/*
  rmdir "$fld"
  rmdir /out
  mv -T /newout /out
fi

shift
cd /out
ls -las
for i; do
  echo "apply patch $i"
  patch -p1 < "$i"
done
