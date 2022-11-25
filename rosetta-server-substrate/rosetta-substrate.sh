#!/bin/bash
/app/substrate --dev &
sleep 15
/app/rosetta-server-substrate &
wait -n
exit $?
