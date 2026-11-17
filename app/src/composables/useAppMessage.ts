import type { MessageApi, MessageOptions, MessageReactive } from "naive-ui";
import type { VNodeChild } from "vue";

type MessageContent = string | (() => VNodeChild);

const DEFAULT_DURATIONS = {
  info: 3000,
  success: 2000,
  warning: 5000,
  error: 30000,
  loading: 0,
};

const DEFAULT_OPTIONS = {
  closable: false,
  keepAliveOnHover: true,
};

// Les stores ne sont pas sous le n-message-provider : AppLayout transmet ici l'API du provider
let messageApi: MessageApi | undefined;

export function registerMessageApi(api: MessageApi) {
  messageApi = api;
}

function getMessageApi() {
  if (messageApi) {
    return messageApi;
  }
  throw new Error(
    "useAppMessage : l'API du n-message-provider n'est pas encore enregistrée par AppLayout"
  );
}

export function useAppMessage() {
  function create(content: MessageContent, options?: MessageOptions): MessageReactive {
    return getMessageApi().create(content, { ...DEFAULT_OPTIONS, ...options });
  }

  function info(content: MessageContent, options?: MessageOptions): MessageReactive {
    return getMessageApi().info(content, {
      ...DEFAULT_OPTIONS,
      duration: DEFAULT_DURATIONS.info,
      ...options,
    });
  }

  function success(content: MessageContent, options?: MessageOptions): MessageReactive {
    return getMessageApi().success(content, {
      ...DEFAULT_OPTIONS,
      duration: DEFAULT_DURATIONS.success,
      ...options,
    });
  }

  function warning(content: MessageContent, options?: MessageOptions): MessageReactive {
    return getMessageApi().warning(content, {
      ...DEFAULT_OPTIONS,
      duration: DEFAULT_DURATIONS.warning,
      ...options,
    });
  }

  function error(content: MessageContent, options?: MessageOptions): MessageReactive {
    return getMessageApi().error(content, {
      ...DEFAULT_OPTIONS,
      closable: true,
      duration: DEFAULT_DURATIONS.error,
      ...options,
    });
  }

  function loading(content: MessageContent, options?: MessageOptions): MessageReactive {
    return getMessageApi().loading(content, {
      ...DEFAULT_OPTIONS,
      duration: DEFAULT_DURATIONS.loading,
      ...options,
    });
  }

  function destroyAll(): void {
    getMessageApi().destroyAll();
  }

  return {
    create,
    info,
    success,
    warning,
    error,
    loading,
    destroyAll,
  };
}
