set datafile separator ","
set title "瞬时频率轨迹"
set xlabel "采样点或帧"
set ylabel "频率 (Hz)"
plot "whatever.csv" with lines title "Instantaneous Frequency"
pause -1
