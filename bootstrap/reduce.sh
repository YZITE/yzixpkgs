set -e
mkdir out && cd out
for i in /*; do
  case "$i" in
    # this is necessary to exclude all directories
    # which would otherwise be unwritable by runc/...
    # and thus the mount would fail of the derived
    # chroot
    (/dev|/home|/mnt|/out|/proc|/sys|/yzix*) ;;
    (/etc|/tmp) cp -rT "$ROOTFS$i" ".$i" ;;
    (*) ln -sT "$ROOTFS$i" ".$i" ;;
  esac
done
