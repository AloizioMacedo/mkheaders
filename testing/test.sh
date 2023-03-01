for f in *; do
  echo "/// Oi galera!\n\n" > tmpfile
  cat $f >> tmpfile
  mv tmpfile $f
done