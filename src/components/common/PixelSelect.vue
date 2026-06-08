<script setup lang="ts">
/**
 * PixelSelect — Custom pixel-styled select dropdown.
 * Renders a custom dropdown instead of a native <select>.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';

export interface SelectOption {
  label: string;
  value: string | number;
}

export interface PixelSelectProps {
  /** Selected value (v-model). */
  modelValue: string | number;
  /** Label text displayed above the select. */
  label?: string;
  /** Array of options. */
  options: SelectOption[];
  /** Error message displayed below the select. */
  error?: string;
  /** Placeholder text when no option is selected. */
  placeholder?: string;
}

const props = withDefaults(defineProps<PixelSelectProps>(), {
  label: '',
  error: '',
  placeholder: '请选择...',
});

const emit = defineEmits<{
  'update:modelValue': [value: string | number];
}>();

const isOpen = ref(false);
const selectRef = ref<HTMLElement | null>(null);

/** The currently selected option object. */
const selectedOption = computed(() =>
  props.options.find((opt) => opt.value === props.modelValue) ?? null
);

/** Display text for the selected option. */
const displayText = computed(() =>
  selectedOption.value ? selectedOption.value.label : props.placeholder
);

/** Whether there's an error. */
const hasError = computed(() => !!props.error);

/** Toggle the dropdown open/close. */
function toggleDropdown(): void {
  isOpen.value = !isOpen.value;
}

/** Select an option and close the dropdown. */
function selectOption(option: SelectOption): void {
  emit('update:modelValue', option.value);
  isOpen.value = false;
}

/** Close the dropdown when clicking outside. */
function handleClickOutside(event: MouseEvent): void {
  if (selectRef.value && !selectRef.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div ref="selectRef" class="pixel-select-wrapper">
    <label v-if="label" class="pixel-select__label">{{ label }}</label>
    <div
      class="pixel-select"
      :class="{ 'pixel-select--open': isOpen, 'pixel-select--error': hasError }"
      @click="toggleDropdown"
    >
      <span class="pixel-select__display" :class="{ 'pixel-select__placeholder': !selectedOption }">
        {{ displayText }}
      </span>
      <span class="pixel-select__arrow" :class="{ 'pixel-select__arrow--open': isOpen }">▼</span>
    </div>

    <!-- Dropdown -->
    <Transition name="dropdown">
      <div v-if="isOpen" class="pixel-select__dropdown">
        <div
          v-for="option in options"
          :key="option.value"
          class="pixel-select__option"
          :class="{ 'pixel-select__option--selected': option.value === modelValue }"
          @click.stop="selectOption(option)"
        >
          {{ option.label }}
        </div>
        <div v-if="options.length === 0" class="pixel-select__empty">
          暂无选项
        </div>
      </div>
    </Transition>

    <p v-if="hasError" class="pixel-select__error">{{ error }}</p>
  </div>
</template>

<style scoped>
.pixel-select-wrapper {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pixel-select__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: 2px;
}

.pixel-select {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-family: var(--font-body);
  font-size: var(--font-size-base);
  color: var(--color-text);
  background: rgba(0, 0, 0, 0.3);
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all var(--transition-fast);
  user-select: none;
}

.pixel-select:hover {
  border-color: var(--color-border-hover);
}

.pixel-select--open {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 1px var(--color-primary);
}

.pixel-select--error {
  border-color: var(--color-danger) !important;
  box-shadow: 0 0 0 1px var(--color-danger) !important;
}

.pixel-select__display {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pixel-select__placeholder {
  color: var(--color-text-muted);
}

.pixel-select__arrow {
  font-size: 10px;
  color: var(--color-text-secondary);
  transition: transform var(--transition-fast);
  margin-left: 8px;
}

.pixel-select__arrow--open {
  transform: rotate(180deg);
}

.pixel-select__dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 100;
  margin-top: 4px;
  background: #1A1A2E;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel);
  max-height: 240px;
  overflow-y: auto;
}

.pixel-select__option {
  padding: 8px 12px;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.pixel-select__option:hover {
  background: var(--color-surface-hover);
  color: var(--color-primary);
}

.pixel-select__option--selected {
  background: rgba(126, 200, 80, 0.1);
  color: var(--color-primary);
  border-left: 2px solid var(--color-primary);
}

.pixel-select__empty {
  padding: 12px;
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  text-align: center;
}

.pixel-select__error {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
  margin-top: 2px;
}

/* Dropdown transition */
.dropdown-enter-active {
  transition: all 0.15s ease-out;
}

.dropdown-leave-active {
  transition: all 0.1s ease-in;
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-4px);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
