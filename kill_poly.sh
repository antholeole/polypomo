kill -9 $(netstat -np 2>/dev/null | grep polypomo | awk '{print $7}')