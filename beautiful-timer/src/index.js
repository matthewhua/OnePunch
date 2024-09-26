let timer
let seconds = 0

function formatTime(seconds) {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hrs.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;

}

function startTimer() {
    if (timer) return; // 防止重复启动
    timer = setInterval(() => {
        seconds++;
        document.getElementById('time-display').textContent = formatTime(seconds);
    }, 1000);

    // 添加动画效果
    document.getElementById('time-display').classList.add('animate-ping');
}


function stopTimer() {
    clearInterval(timer);
    timer = null;

    // 移除动画效果
    document.getElementById('time-display').classList.remove('animate-ping');
}

function resetTimer() {
    stopTimer();
    seconds = 0;
    document.getElementById('time-display').textContent = formatTime(seconds);
}

document.getElementById('start-btn').addEventListener('click', startTimer);
document.getElementById('stop-btn').addEventListener('click', stopTimer);
document.getElementById('reset-btn').addEventListener('click', resetTimer);