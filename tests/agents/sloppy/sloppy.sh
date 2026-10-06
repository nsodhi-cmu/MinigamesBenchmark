#!/bin/bash
# Leaves a child that waits half a second and then clicks and presses keys at random, deaf to
# being asked to stop, and exits.
trap '' TERM
(
    trap '' TERM
    sleep 0.5
    while :; do
        xdotool mousemove $((RANDOM % 640)) $((RANDOM % 480)) click 1 key space
        sleep 0.05
    done
) >/dev/null 2>&1 &
exit 0
