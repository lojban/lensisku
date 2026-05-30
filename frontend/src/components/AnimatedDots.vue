<template>
  <span class="animated-dots" aria-hidden="true" role="presentation">
    <span class="dot" />
    <span class="dot" />
    <span class="dot" />
  </span>
</template>

<style scoped>
.animated-dots {
  display: inline-flex;
  align-items: center;
  gap: 0.15em;
  margin-left: 0.15em;
  vertical-align: baseline;
}

.dot {
  width: 0.3em;
  height: 0.3em;
  border-radius: 50%;
  background-color: currentColor;
  animation: dot-pulse 1.4s ease-in-out infinite both;
}

.dot:nth-child(2) {
  animation-delay: 0.16s;
}

.dot:nth-child(3) {
  animation-delay: 0.32s;
}

@keyframes dot-pulse {
  0%,
  80%,
  100% {
    opacity: 0.25;
    transform: scale(0.8);
  }
  40% {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
