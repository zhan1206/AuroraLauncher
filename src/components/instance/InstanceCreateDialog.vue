<script setup lang="ts">
/**
 * InstanceCreateDialog — Dialog for creating a new game instance.
 */
import { ref, computed } from 'vue';
import PixelDialog from '@/components/common/PixelDialog.vue';
import PixelInput from '@/components/common/PixelInput.vue';
import VersionSelector from '@/components/version/VersionSelector.vue';
import LoaderSelector from '@/components/version/LoaderSelector.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import { useInstanceStore } from '@/stores/instance';
import { validateInstanceName } from '@/utils/validators';
import type { LoaderType } from '@/types/instance';
import { useToast } from '@/components/common/useToast';

export interface InstanceCreateDialogProps {
  /** Whether the dialog is visible. */
  modelValue: boolean;
}

defineProps<InstanceCreateDialogProps>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  created: [];
}>();

const instanceStore = useInstanceStore();
const toast = useToast();

// Form state
const name = ref('');
const versionId = ref('');
const loaderType = ref<LoaderType>('Vanilla');
const creating = ref(false);

// Validation
const nameError = computed(() => {
  if (!name.value) return '';
  const result = validateInstanceName(name.value);
  return result.valid ? '' : result.message;
});

const versionError = computed(() => {
  if (!versionId.value && name.value) return '请选择游戏版本';
  return '';
});

const canCreate = computed(() =>
  name.value.trim().length > 0 &&
  versionId.value.length > 0 &&
  !nameError.value &&
  !creating.value
);

/** Create the instance. */
async function handleCreate(): Promise<void> {
  if (!canCreate.value) return;

  creating.value = true;
  try {
    const result = await instanceStore.createInstance({
      name: name.value.trim(),
      version_id: versionId.value,
      loader_type: loaderType.value,
    });

    if (result) {
      toast.success(`实例 "${result.name}" 创建成功`);
      emit('created');
      handleClose();
    }
  } catch (e) {
    toast.error('创建实例失败');
  } finally {
    creating.value = false;
  }
}

/** Close the dialog and reset form. */
function handleClose(): void {
  name.value = '';
  versionId.value = '';
  loaderType.value = 'Vanilla';
  emit('update:modelValue', false);
}
</script>

<template>
  <PixelDialog
    :model-value="modelValue"
    title="新建实例"
    width="560px"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="create-form">
      <PixelInput
        v-model="name"
        label="实例名称"
        placeholder="输入实例名称..."
        :error="nameError"
      />

      <div class="create-form__section">
        <label class="create-form__label">游戏版本</label>
        <VersionSelector v-model="versionId" />
        <p v-if="versionError" class="create-form__error">{{ versionError }}</p>
      </div>

      <div class="create-form__section">
        <label class="create-form__label">加载器类型</label>
        <LoaderSelector v-model="loaderType" />
      </div>
    </div>

    <template #footer>
      <PixelButton variant="ghost" @click="handleClose">取消</PixelButton>
      <PixelButton
        variant="primary"
        :loading="creating"
        :disabled="!canCreate"
        @click="handleCreate"
      >
        创建实例
      </PixelButton>
    </template>
  </PixelDialog>
</template>

<style scoped>
.create-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.create-form__section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.create-form__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.create-form__error {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
}
</style>
