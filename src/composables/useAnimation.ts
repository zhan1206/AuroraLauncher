/**
 * Animation composable for Aurora Launcher.
 * Provides stagger animations, bounce effects, and transition helpers.
 */
import { ref, onMounted, nextTick, type Ref } from 'vue';

/** Options for stagger animation. */
export interface StaggerOptions {
  /** Delay between each item in milliseconds (default: 50). */
  delay?: number;
  /** Initial delay before the first item starts (default: 0). */
  initialDelay?: number;
  /** Animation duration for each item in milliseconds (default: 350). */
  duration?: number;
}

/**
 * Composable that provides staggered fade-in animation for a list of items.
 *
 * @param count - Number of items (or a ref to the count).
 * @param options - Stagger animation options.
 * @returns An object with `visibleCount` (ref) and `startAnimation` method.
 */
export function useStaggerAnimation(
  count: Ref<number> | number,
  options: StaggerOptions = {}
): {
  visibleCount: Ref<number>;
  startAnimation: () => void;
  reset: () => void;
} {
  const { delay = 50, initialDelay = 0, duration = 350 } = options;
  const visibleCount = ref(0);
  let timerIds: ReturnType<typeof setTimeout>[] = [];

  function startAnimation(): void {
    reset();
    const total = typeof count === 'number' ? count : count.value;

    const initialTimer = setTimeout(() => {
      for (let i = 0; i < total; i++) {
        const timer = setTimeout(() => {
          visibleCount.value = i + 1;
        }, i * delay);
        timerIds.push(timer);
      }
      // Final cleanup timer
      const cleanupTimer = setTimeout(() => {
        visibleCount.value = total;
      }, total * delay + duration);
      timerIds.push(cleanupTimer);
    }, initialDelay);
    timerIds.push(initialTimer);
  }

  function reset(): void {
    for (const id of timerIds) {
      clearTimeout(id);
    }
    timerIds = [];
    visibleCount.value = 0;
  }

  return { visibleCount, startAnimation, reset };
}

/**
 * Composable for a simple boolean toggle with optional animation class.
 *
 * @param initialValue - Initial toggle state (default: false).
 * @returns An object with `isOn`, `toggle`, `turnOn`, `turnOff` methods.
 */
export function useToggle(initialValue: boolean = false): {
  isOn: Ref<boolean>;
  toggle: () => void;
  turnOn: () => void;
  turnOff: () => void;
} {
  const isOn = ref(initialValue);

  function toggle(): void {
    isOn.value = !isOn.value;
  }

  function turnOn(): void {
    isOn.value = true;
  }

  function turnOff(): void {
    isOn.value = false;
  }

  return { isOn, toggle, turnOn, turnOff };
}

/**
 * Composable for a bounce animation on a target element.
 * Triggers a pixel-bounce CSS animation on demand.
 *
 * @returns An object with `isBouncing` ref and `trigger` method.
 */
export function useBounce(): {
  isBouncing: Ref<boolean>;
  trigger: () => void;
} {
  const isBouncing = ref(false);

  function trigger(): void {
    if (isBouncing.value) return;
    isBouncing.value = true;
    setTimeout(() => {
      isBouncing.value = false;
    }, 600);
  }

  return { isBouncing, trigger };
}

/**
 * Composable for fade-in on mount.
 * Automatically sets `visible` to true after the component mounts.
 *
 * @param delay - Delay in ms before becoming visible (default: 0).
 * @returns An object with `visible` ref.
 */
export function useFadeIn(delay: number = 0): {
  visible: Ref<boolean>;
} {
  const visible = ref(false);

  onMounted(() => {
    if (delay > 0) {
      setTimeout(() => {
        visible.value = true;
      }, delay);
    } else {
      nextTick(() => {
        visible.value = true;
      });
    }
  });

  return { visible };
}

/**
 * Composable for counting up a number with animation.
 *
 * @param target - Target number (or ref).
 * @param duration - Animation duration in ms (default: 1000).
 * @returns An object with `current` ref and `start` method.
 */
export function useCountUp(
  target: Ref<number> | number,
  duration: number = 1000
): {
  current: Ref<number>;
  start: () => void;
} {
  const current = ref(0);
  let animFrame: number | null = null;

  function start(): void {
    if (animFrame !== null) {
      cancelAnimationFrame(animFrame);
    }
    const targetValue = typeof target === 'number' ? target : target.value;
    const startTime = performance.now();
    const startValue = current.value;

    function step(now: number): void {
      const elapsed = now - startTime;
      const progress = Math.min(elapsed / duration, 1);
      // Ease-out cubic
      const eased = 1 - Math.pow(1 - progress, 3);
      current.value = Math.round(startValue + (targetValue - startValue) * eased);

      if (progress < 1) {
        animFrame = requestAnimationFrame(step);
      } else {
        current.value = targetValue;
        animFrame = null;
      }
    }

    animFrame = requestAnimationFrame(step);
  }

  return { current, start };
}

/**
 * useStagger — Provides staggered fade-in animation for a list of items.
 * Returns CSS class bindings for each item to animate in sequence.
 *
 * @param items - Reactive array of items to animate.
 * @param delay - Delay between each item in ms (default: 50).
 * @returns An object with `visibleCount` ref, `isItemVisible` checker, and `startAnimation` method.
 */
export function useStagger(
  items: Ref<any[]>,
  delay: number = 50
): {
  visibleCount: Ref<number>;
  isItemVisible: (index: number) => boolean;
  startAnimation: () => void;
  reset: () => void;
} {
  const visibleCount = ref(0);
  let timerIds: ReturnType<typeof setTimeout>[] = [];

  function isItemVisible(index: number): boolean {
    return index < visibleCount.value;
  }

  function startAnimation(): void {
    reset();
    const total = items.value.length;

    for (let i = 0; i < total; i++) {
      const timer = setTimeout(() => {
        visibleCount.value = i + 1;
      }, i * delay);
      timerIds.push(timer);
    }
  }

  function reset(): void {
    for (const id of timerIds) {
      clearTimeout(id);
    }
    timerIds = [];
    visibleCount.value = 0;
  }

  return { visibleCount, isItemVisible, startAnimation, reset };
}

/**
 * useGlowPulse — Controls a glow-pulse animation on an element.
 *
 * @param active - Ref controlling whether the glow animation is active.
 * @returns An object with `glowClass` computed for binding to the element.
 */
export function useGlowPulse(active: Ref<boolean>): {
  glowClass: Ref<string>;
  setActive: (value: boolean) => void;
} {
  const glowClass = ref('');

  function setActive(value: boolean): void {
    active.value = value;
    glowClass.value = value ? 'animate-glow-pulse' : '';
  }

  // Initialize class based on initial active state
  if (active.value) {
    glowClass.value = 'animate-glow-pulse';
  }

  return { glowClass, setActive };
}
