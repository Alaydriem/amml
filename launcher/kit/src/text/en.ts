import { aboutSettingsEn } from "./en/aboutSettings";
import { addModBrowseEn } from "./en/addModBrowse";
import { addModUrlEn } from "./en/addModUrl";
import { dialogsEn } from "./en/dialogs";
import { downloadsSettingsEn } from "./en/downloadsSettings";
import { faultsEn } from "./en/faults";
import { firstRunAdoptEn } from "./en/firstRunAdopt";
import { firstRunChecksEn } from "./en/firstRunChecks";
import { gameVersionsSettingsEn } from "./en/gameVersionsSettings";
import { installProgressEn } from "./en/installProgress";
import { instanceDetailEn } from "./en/instanceDetail";
import { instanceHeaderEn, instanceMenuEn, instanceRowEn, instancesEn } from "./en/instances";
import { newInstanceEn } from "./en/newInstance";
import { registriesSettingsEn } from "./en/registriesSettings";
import { sharedDataSettingsEn } from "./en/sharedDataSettings";
import { shellEn } from "./en/shell";
import { unitsEn } from "./en/units";

/**
 * Every English text of the kit, one key per screen or shared part. The app's i18n step builds a value of the same
 * type from its catalog and passes each part to its screen.
 */
export const en = {
    shell: shellEn,
    units: unitsEn,
    firstRunChecks: firstRunChecksEn,
    firstRunAdopt: firstRunAdoptEn,
    instances: instancesEn,
    instanceRow: instanceRowEn,
    instanceMenu: instanceMenuEn,
    instanceHeader: instanceHeaderEn,
    newInstance: newInstanceEn,
    installProgress: installProgressEn,
    instanceDetail: instanceDetailEn,
    addModBrowse: addModBrowseEn,
    addModUrl: addModUrlEn,
    registriesSettings: registriesSettingsEn,
    gameVersionsSettings: gameVersionsSettingsEn,
    sharedDataSettings: sharedDataSettingsEn,
    downloadsSettings: downloadsSettingsEn,
    aboutSettings: aboutSettingsEn,
    dialogs: dialogsEn,
    faults: faultsEn,
};

export type LauncherText = typeof en;
