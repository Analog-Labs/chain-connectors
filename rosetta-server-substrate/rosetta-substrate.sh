#!/bin/bash
/app/substrate --dev &
sleep 5
/app/rosetta-server-substrate &
wait -n
exit $?
