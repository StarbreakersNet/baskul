<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HCollapse from "@/components/c/hCollapse.vue";
import HCollapseItem from "@/components/c/hCollapseItem.vue";
import HEmpty from "@/components/c/hEmpty.vue";
import HText from "@/components/c/hText.vue";
import HTime from "@/components/c/hTime.vue";
import HTimeline from "@/components/c/hTimeline.vue";
import HTimelineItem from "@/components/c/hTimelineItem.vue";
import { useCurrentTime } from "@/composables/useCurrentTime";
import { useActivityStore } from "@/stores/activity";
import { describeActivity, groupByDay } from "@/utils/activity";
import { capitalize } from "@/utils/text";

const CLOCK_MS = 15_000;
const DAY_MS = 24 * 60 * 60 * 1000;

const activity = useActivityStore();
const { t, locale } = useI18n();
const now = useCurrentTime(CLOCK_MS);

const days = computed(() =>
  groupByDay(activity.events).map(group => ({
    ...group,
    title: dayTitle(group.day),
    items: group.events.map(event => ({ ...event, label: describeActivity(event, locale.value) })),
  }))
);
const firstDay = computed(() => (days.value.length ? [days.value[0].day] : []));

function dayTitle(day: number) {
  const today = new Date(now.value);
  today.setHours(0, 0, 0, 0);
  let title = capitalize(
    new Intl.DateTimeFormat(locale.value, {
      weekday: "long",
      day: "numeric",
      month: "long",
    }).format(day)
  );
  if (day === today.getTime()) {
    title = t("home.today");
  } else if (day === today.getTime() - DAY_MS) {
    title = t("home.yesterday");
  }
  return title;
}
</script>

<template>
  <h-card size="small" :title="$t('home.activity')">
    <template #header-extra>
      <transition name="activity-fade">
        <h-button v-if="activity.events.length" text type="primary" @click="activity.clear()">
          {{ $t("home.activityClear") }}
        </h-button>
      </transition>
    </template>
    <transition mode="out-in" name="activity-fade">
      <div v-if="days.length" key="days" class="activity-days">
        <h-collapse :default-expanded-names="firstDay">
          <h-collapse-item
            v-for="group in days"
            :key="group.day"
            :name="group.day"
            :title="group.title">
            <template #header-extra>
              <h-text depth="3">{{ group.items.length }}</h-text>
            </template>
            <h-timeline>
              <transition-group name="activity-event">
                <h-timeline-item
                  v-for="item in group.items"
                  :key="item.id"
                  :type="item.label.color">
                  <template #header>
                    {{ $t(`home.${item.label.key}`, item.label.params) }}
                  </template>
                  <template #footer>
                    <transition mode="out-in" name="activity-fade">
                      <span v-if="item.count > 1" key="run">
                        <h-time format="HH:mm" :time="item.at" />
                        →
                        <h-time format="HH:mm" :time="item.until" />
                      </span>
                      <h-time
                        v-else
                        key="single"
                        :time="item.at"
                        :to="Math.max(now, item.at)"
                        type="relative" />
                    </transition>
                  </template>
                </h-timeline-item>
              </transition-group>
            </h-timeline>
          </h-collapse-item>
        </h-collapse>
      </div>
      <h-empty v-else key="empty" class="activity-empty" :description="$t('home.activityEmpty')" />
    </transition>
  </h-card>
</template>

<style lang="scss" scoped>
.activity-days {
  max-height: 320px;
  overflow-y: auto;
  padding-right: 4px;
}

.activity-empty {
  padding: 16px 0;
}

.activity-fade-enter-active,
.activity-fade-leave-active {
  transition: opacity 0.2s ease;
}

.activity-fade-enter-from,
.activity-fade-leave-to {
  opacity: 0;
}

.activity-event-enter-active {
  transition:
    opacity 0.3s ease,
    transform 0.3s ease;
}

.activity-event-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}

@media (prefers-reduced-motion: reduce) {
  .activity-fade-enter-active,
  .activity-fade-leave-active,
  .activity-event-enter-active {
    transition: none;
  }
}
</style>
