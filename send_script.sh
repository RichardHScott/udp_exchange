#!/bin/bash

uuid='deadbeef-dead-dead-dead-beefbeefbeef'

msg='{'

msg+='"disk": ['
first_line=1
while read -r line; do
	if [ $first_line -eq 1 ]; then
		first_line=0
	else 
		msg+=','
	fi

	msg+=$line
done < <(df | awk '/%/ && NR > 1 {print "{\"use\":\"" $5 "\", \"mount\":\"" $6 "\"}" }')
msg+=']'

msg+='}'
echo $msg

`./udp_exchange client "127.0.0.1:5890" "$uuid" "$msg"`