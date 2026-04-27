import * as semver from "semver";
import * as z from "zod";

import { invoke } from "./invoke";
import useLocalStorage from "./useLocalStorage.svelte";

interface LatestVersionInfo {
  version: string;
}

const fetchLatestVersion = async (): Promise<LatestVersionInfo> => {
  const response = await fetch(
    "https://files.radicle.dev/releases/radicle-desktop/latest/latest.json",
  );
  const body: LatestVersionInfo = await response.json();
  return body;
};

// Check for new version every hour.
const VERSION_CHECK_INTERVAL = 3600 * 1000;

const isEnabledStore = useLocalStorage(
  "updateChecker.isEnabled",
  z.boolean().nullable(),
  null,
  !window.localStorage,
);

class UpdateChecker {
  private checkInterval: number | undefined = $state();
  private latestVersionInfo: LatestVersionInfo | undefined = $state();
  private sanitizedCurrentVersion: string | undefined = $state();
  public currentVersion: string | undefined = $state();

  public isEnabled = $derived.by(() => {
    if (isEnabledStore.value === null) {
      return false;
    } else {
      return isEnabledStore.value;
    }
  });

  // A state that holds the `LatestVersionInfo` if this feature has
  // been enabled and if there is a newer version available.
  public newVersion = $derived.by(() => {
    if (this.latestVersionInfo && this.sanitizedCurrentVersion) {
      if (
        semver.gt(this.latestVersionInfo.version, this.sanitizedCurrentVersion)
      ) {
        return this.latestVersionInfo.version;
      } else {
        return undefined;
      }
    } else {
      return undefined;
    }
  });

  public static init(): UpdateChecker {
    const updateChecker = new UpdateChecker();

    void invoke<string>("version").then(currentVersion => {
      updateChecker.currentVersion = currentVersion.toString();
      const version = semver.coerce(currentVersion);
      if (version) {
        updateChecker.sanitizedCurrentVersion = version.toString();
      }
      if (isEnabledStore.value) {
        updateChecker.enable();
      }
    });

    return updateChecker;
  }

  // Disable background update checking.
  public disable = (): void => {
    isEnabledStore.value = false;

    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = undefined;
    }
  };

  private async checkNewVersion(): Promise<boolean> {
    try {
      this.latestVersionInfo = await fetchLatestVersion();
    } catch {
      return false;
    }

    return (
      this.sanitizedCurrentVersion !== undefined &&
      semver.gt(this.latestVersionInfo.version, this.sanitizedCurrentVersion)
    );
  }

  // Enable background udpate checking.
  public enable = (): void => {
    isEnabledStore.value = true;

    void this.checkNewVersion();
    if (!this.checkInterval) {
      this.checkInterval = window.setInterval(() => {
        void this.checkNewVersion();
      }, VERSION_CHECK_INTERVAL);
    }
  };
}

export const updateChecker = UpdateChecker.init();
