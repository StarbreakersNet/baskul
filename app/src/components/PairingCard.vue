<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onDeactivated, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { IconCopy, IconEye, IconEyeOff, IconKey } from "@tabler/icons-vue";
import HAlert from "@/components/c/hAlert.vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HCollapse from "@/components/c/hCollapse.vue";
import HCollapseItem from "@/components/c/hCollapseItem.vue";
import HDescriptions from "@/components/c/hDescriptions.vue";
import HDescriptionsItem from "@/components/c/hDescriptionsItem.vue";
import HFlex from "@/components/c/hFlex.vue";
import HIcon from "@/components/c/hIcon.vue";
import HInput from "@/components/c/hInput.vue";
import HInputOtp from "@/components/c/hInputOtp.vue";
import HSelect from "@/components/c/hSelect.vue";
import HText from "@/components/c/hText.vue";
import { useCopy } from "@/composables/useCopy";
import { useCurrentTime } from "@/composables/useCurrentTime";
import { useNetworkStore } from "@/stores/network";
import { usePairingStore } from "@/stores/pairing";
import { useSettingsStore } from "@/stores/settings";
import { hostWithoutDefaultPort } from "@/utils/address";
import { osLabel } from "@/utils/os";

const { isPrimary } = defineProps<{ isPrimary: boolean }>();

const CODE_LENGTH = 6;
const SEARCH_MS = 3000;

const settings = useSettingsStore();
const pairing = usePairingStore();
const networkStore = useNetworkStore();
const router = useRouter();
const { copy } = useCopy();
const { t } = useI18n();
const now = useCurrentTime(1000);

const showSecret = ref(false);
const code = ref<string[]>([]);
const typedAddress = ref("");
const chosenAddress = ref<string | null>(null);

const address = computed(() => {
  const ip = settings.info?.address;
  const port = settings.config?.network.listen.split(":").pop();
  return ip && port ? `${ip}:${port}` : (ip ?? "");
});
const secret = computed(() => settings.config?.network.secret ?? "");
const maskedSecret = computed(() => "•".repeat(Math.min(secret.value.length, 16)));

const waitingCode = computed(() =>
  pairing.state.state === "waiting"
    ? `${pairing.state.code.slice(0, 3)} ${pairing.state.code.slice(3)}`
    : ""
);
const remaining = computed(() => {
  const seconds =
    pairing.state.state === "waiting"
      ? Math.max(0, Math.round((pairing.state.expires - now.value) / 1000))
      : 0;
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
});

const primaryOptions = computed(() =>
  (networkStore.network?.primaries ?? [])
    .filter(primary => primary.protocol === networkStore.network?.protocol)
    .map(primary => {
      const host = hostWithoutDefaultPort(
        primary.addresses[0] ?? "",
        networkStore.network?.defaultPort
      );
      return { label: `${primary.name} · ${osLabel(primary.os)} · ${host}`, value: host };
    })
);
const joinAddress = computed(
  () => chosenAddress.value ?? primaryOptions.value[0]?.value ?? typedAddress.value.trim()
);
const canJoin = computed(
  () => code.value.join("").length === CODE_LENGTH && joinAddress.value !== ""
);

let searchTimer: ReturnType<typeof setInterval> | undefined;

function copyManual() {
  void copy(
    t("home.pairingClipboard", { address: address.value, secret: secret.value }),
    "home.pairingCopied"
  );
}

function join() {
  void pairing.join(joinAddress.value, code.value.join(""));
}

function openConnection() {
  router.push({ name: "settings", query: { onglet: "connexion" } });
}

function startSearching() {
  stopSearching();
  if (!isPrimary) {
    void networkStore.refresh();
    searchTimer = setInterval(networkStore.refresh, SEARCH_MS);
  }
}

function stopSearching() {
  clearInterval(searchTimer);
}

onActivated(startSearching);
onDeactivated(stopSearching);
onBeforeUnmount(stopSearching);
</script>

<template>
  <h-card size="small" :title="$t('home.pairing')">
    <transition mode="out-in" name="fade">
      <h-flex v-if="isPrimary" key="primary" :size="12" vertical>
        <h-text depth="3">{{ $t("pairing.primaryText") }}</h-text>
        <transition mode="out-in" name="fade">
          <h-flex
            v-if="pairing.state.state === 'waiting'"
            key="waiting"
            align="center"
            :size="6"
            vertical>
            <h-text class="pairing-code mono">{{ waitingCode }}</h-text>
            <h-text class="small-text" depth="3">
              {{ $t("pairing.validFor", { time: remaining }) }}
            </h-text>
            <h-button size="small" @click="pairing.cancel()">{{ $t("common.cancel") }}</h-button>
          </h-flex>
          <h-flex v-else key="other" :size="10" vertical>
            <transition mode="out-in" name="fade">
              <h-alert
                v-if="pairing.state.state === 'paired'"
                key="paired"
                :bordered="false"
                type="success">
                {{ $t("pairing.paired", { name: pairing.state.name }) }}
              </h-alert>
              <h-alert
                v-else-if="pairing.state.state === 'failed'"
                key="failed"
                :bordered="false"
                type="warning">
                {{ pairing.state.reason }}
              </h-alert>
            </transition>
            <div>
              <h-button :loading="pairing.starting.loading" type="primary" @click="pairing.start()">
                <template #icon>
                  <h-icon><icon-key /></h-icon>
                </template>
                {{ $t("pairing.showCode") }}
              </h-button>
            </div>
          </h-flex>
        </transition>
        <h-collapse>
          <h-collapse-item name="manual" :title="$t('pairing.manual')">
            <h-flex :size="8" vertical>
              <h-descriptions :column="1" label-placement="left" size="small">
                <h-descriptions-item :label="$t('home.pairingAddress')">
                  <h-text code>{{ address || "—" }}</h-text>
                </h-descriptions-item>
                <h-descriptions-item :label="$t('home.pairingSecret')">
                  <h-flex align="center" :size="4" :wrap="false">
                    <h-text code>{{ showSecret ? secret : maskedSecret }}</h-text>
                    <h-button
                      :aria-label="
                        showSecret ? $t('home.pairingHideSecret') : $t('home.pairingShowSecret')
                      "
                      quaternary
                      size="tiny"
                      @click="showSecret = !showSecret">
                      <template #icon>
                        <h-icon :component="showSecret ? IconEyeOff : IconEye" />
                      </template>
                    </h-button>
                  </h-flex>
                </h-descriptions-item>
              </h-descriptions>
              <div>
                <h-button secondary size="small" @click="copyManual()">
                  <template #icon>
                    <h-icon><icon-copy /></h-icon>
                  </template>
                  {{ $t("home.pairingCopy") }}
                </h-button>
              </div>
            </h-flex>
          </h-collapse-item>
        </h-collapse>
      </h-flex>

      <h-flex v-else key="secondary" :size="12" vertical>
        <h-text depth="3">{{ $t("pairing.secondaryText") }}</h-text>
        <transition mode="out-in" name="fade">
          <h-select
            v-if="primaryOptions.length"
            key="found"
            v-model:value="chosenAddress"
            :options="primaryOptions"
            :placeholder="primaryOptions[0]?.label" />
          <h-input
            v-else
            key="typed"
            v-model:value="typedAddress"
            class="mono"
            :input-props="{ spellcheck: false }"
            :placeholder="$t('pairing.addressPlaceholder')" />
        </transition>
        <h-input-otp v-model:value="code" :length="CODE_LENGTH" />
        <h-flex align="center" :size="8">
          <h-button
            :disabled="!canJoin"
            :loading="pairing.joining.loading"
            type="primary"
            @click="join()">
            {{ $t("pairing.join") }}
          </h-button>
          <h-button quaternary @click="openConnection()">
            {{ $t("home.pairingCheck") }}
          </h-button>
        </h-flex>
      </h-flex>
    </transition>
  </h-card>
</template>

<style lang="scss" scoped>
.pairing-code {
  font-size: 34px;
  font-weight: 700;
  letter-spacing: 0.12em;
}
</style>
