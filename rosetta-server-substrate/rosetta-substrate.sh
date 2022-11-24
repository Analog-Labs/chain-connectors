#!/bin/bash
/app/substrate --dev &
sleep 10
/app/rosetta-server-substrate &
wait -n
exit $?
