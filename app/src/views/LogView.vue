<script setup lang="ts">
import {
  computed,
  nextTick,
  onActivated,
  onBeforeUnmount,
  onDeactivated,
  ref,
  useTemplateRef,
  watch,
} from "vue";
import { IconCopy, IconSearch } from "@tabler/icons-vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HEmpty from "@/components/c/hEmpty.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HInput from "@/components/c/hInput.vue";
import HLog from "@/components/c/hLog.vue";
import HSwitch from "@/components/c/hSwitch.vue";
import HTag from "@/components/c/hTag.vue";
import HText from "@/components/c/hText.vue";
import { useCopy } from "@/composables/useCopy";
import { useLogStore } from "@/stores/log";
import {
  filterLogLines,
  LOG_LANGUAGE,
  LOG_LEVELS,
  logHighlighter,
  parseLogLines,
  type LogLevel,
} from "@/utils/logLines";
import type { HLogInst } from "@/types/components/c/hLog";

const REFRESH_MS = 2000;
const LEVEL_KEYS: Record<LogLevel, string> = {
  error: "log.levelError",
  warn: "log.levelWarn",
  info: "log.levelInfo",
  debug: "log.levelDebug",
};
const LEVEL_TYPES: Record<LogLevel, "error" | "warning" | "success" | "default"> = {
  error: "error",
  warn: "warning",
  info: "success",
  debug: "default",
};

const logStore = useLogStore();
const { copy } = useCopy();

const isFollowing = ref(true);
const search = ref("");
const levels = ref<LogLevel[]>([...LOG_LEVELS]);
const logRef = useTemplateRef<HLogInst>("logRef");

const lines = computed(() => parseLogLines(logStore.text));
const visibleLines = computed(() => filterLogLines(lines.value, levels.value, search.value));
const visibleText = computed(() => visibleLines.value.map(line => line.text).join("\n"));
const levelCounts = computed(() => {
  const counts: Record<LogLevel, number> = { error: 0, warn: 0, info: 0, debug: 0 };
  for (const line of lines.value) {
    counts[line.level] += 1;
  }
  return counts;
});
const isFiltered = computed(() => !!search.value.trim() || levels.value.length < LOG_LEVELS.length);

let refreshTimer: ReturnType<typeof setInterval> | undefined;

function toggleLevel(level: LogLevel, isChecked: boolean) {
  levels.value = isChecked
    ? LOG_LEVELS.filter(item => item === level || levels.value.includes(item))
    : levels.value.filter(item => item !== level);
}

async function scrollToEnd() {
  if (isFollowing.value) {
    await nextTick();
    logRef.value?.scrollToBottom();
  }
}

async function refresh() {
  if (await logStore.refresh()) {
    await scrollToEnd();
  }
}

function startRefreshing() {
  clearInterval(refreshTimer);
  void refresh();
  refreshTimer = setInterval(refresh, REFRESH_MS);
}

function stopRefreshing() {
  clearInterval(refreshTimer);
}

function copyVisible() {
  void copy(visibleText.value, "log.copied");
}

watch(visibleText, scrollToEnd);

onActivated(startRefreshing);
onDeactivated(stopRefreshing);
onBeforeUnmount(stopRefreshing);
</script>

<template>
  <h-card size="small" :title="$t('log.title')">
    <template #header-extra>
      <h-flex align="center" :size="12">
        <h-flex align="center" :size="6">
          <h-switch v-model:value="isFollowing" size="small" />
          <h-text depth="3">{{ $t("log.follow") }}</h-text>
        </h-flex>
        <h-button size="small" @click="logStore.openFile()">{{ $t("common.openFile") }}</h-button>
      </h-flex>
    </template>

    <h-flex align="center" class="log-toolbar" justify="space-between" :size="12">
      <h-flex align="center" :size="6">
        <h-tag
          v-for="level in LOG_LEVELS"
          :key="level"
          checkable
          :checked="levels.includes(level)"
          class="log-level"
          :type="LEVEL_TYPES[level]"
          @update:checked="toggleLevel(level, $event)">
          {{ $t(LEVEL_KEYS[level]) }}
          <span class="log-level-count">{{ levelCounts[level] }}</span>
        </h-tag>
      </h-flex>
      <h-input
        v-model:value="search"
        class="log-search"
        clearable
        :placeholder="$t('log.search')"
        size="small">
        <template #prefix>
          <h-icon :component="IconSearch" />
        </template>
      </h-input>
    </h-flex>

    <transition mode="out-in" name="fade">
      <h-empty
        v-if="logStore.isLoaded && lines.length && !visibleLines.length"
        class="log-empty"
        :description="$t('log.noMatch')" />
      <h-log
        v-else
        ref="logRef"
        :hljs="logHighlighter"
        :language="LOG_LANGUAGE"
        :loading="!logStore.isLoaded"
        :log="visibleText"
        :rows="24"
        trim />
    </transition>

    <template #footer>
      <h-flex align="center" justify="space-between" :size="12" :wrap="false">
        <h-text class="mono break-all log-path" depth="3">{{ logStore.path }}</h-text>
        <h-flex align="center" :size="8" :wrap="false">
          <h-text class="log-count" depth="3">
            {{
              isFiltered
                ? $t("log.lineCountFiltered", { n: visibleLines.length, total: lines.length })
                : $t("log.lineCount", { n: lines.length }, lines.length)
            }}
          </h-text>
          <h-button :disabled="!visibleLines.length" size="small" @click="copyVisible()">
            <template #icon>
              <h-icon><icon-copy /></h-icon>
            </template>
            {{ $t("log.copyVisible") }}
          </h-button>
        </h-flex>
      </h-flex>
    </template>
  </h-card>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.log-toolbar {
  margin-bottom: 12px;
}

.log-level {
  font-variant-numeric: tabular-nums;
}

.log-level-count {
  margin-left: 6px;
  opacity: 0.65;
}

.log-search {
  width: 240px;
}

.log-empty {
  padding: 48px 0;
}

.log-path {
  font-size: variables.$font-size-small;
}

.log-count {
  font-size: variables.$font-size-small;
  white-space: nowrap;
}
</style>
