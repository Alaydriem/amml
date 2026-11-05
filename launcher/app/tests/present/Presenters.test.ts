import { afterEach, describe, expect, it } from "vitest";
import type { ModCard } from "@amml/launcher-kit";
import type { CardView } from "#lib/bindings/CardView.ts";
import type { CatalogLayoutView } from "#lib/bindings/CatalogLayoutView.ts";
import type { CheckView } from "#lib/bindings/CheckView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import type { ModView } from "#lib/bindings/ModView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { CatalogPresenter } from "#lib/present/CatalogPresenter.ts";
import { CheckPresenter } from "#lib/present/CheckPresenter.ts";
import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
import { InstancePresenter } from "#lib/present/InstancePresenter.ts";
import { ModPresenter } from "#lib/present/ModPresenter.ts";
import { RegistryPresenter } from "#lib/present/RegistryPresenter.ts";
import { SwatchRules } from "#lib/present/SwatchRules.ts";
import { ModNoteTextSet } from "#lib/text/ModNoteTextSet.ts";

const instance: InstanceView = {
    name: "1.26.60.05",
    folder: "C:\\Users\\charl\\AppData\\Local\\amml\\instances\\1.26.60.05",
    game: "1.26.60.05",
    preview: false,
    ammlVersion: null,
    ammlLocal: false,
    ammlInstalled: true,
    modCount: 4,
    dataMode: "shared",
    dataDir: null,
    managed: true,
    runningSince: null,
    lastPlayed: null,
    updateCount: 1,
};

function mod(overrides: Partial<ModView>): ModView {
    return {
        id: "@alaydriem/peek",
        title: "Peek",
        origin: "registry",
        registryBase: "https://alaydriem.github.io/amml-registry/",
        enabled: true,
        installedVersion: "1.0.0",
        availableVersion: "1.1.0",
        api: 2,
        update: "available",
        ...overrides,
    };
}

const official: RegistryView = {
    id: "abc",
    base: "https://alaydriem.github.io/amml-registry/",
    name: "Official",
    official: true,
    status: "ok",
    modCount: 12,
    fetched: 1_000_000,
};

function card(id: string): ModCard {
    return {
        id,
        registryId: "r",
        title: id,
        summary: "",
        tags: [],
        version: "1.0.0",
        size: 1,
        swatch: 1,
        state: "available",
    };
}

describe("SwatchRules", () => {
    it("gives the same swatch for the same name and spreads names over all ten", () => {
        expect(SwatchRules.forName("1.26.60.05")).toBe(SwatchRules.forName("1.26.60.05"));
        const used = new Set<number>();
        for (let index = 0; index < 400; index += 1) used.add(SwatchRules.forName(`instance-${index}`));
        expect([...used].sort((a, b) => a - b)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    });
});

describe("InstancePresenter", () => {
    afterEach(() => I18n.adopt(null));

    it("shows a folder under the amml root with %LOCALAPPDATA%", () => {
        const shown = InstancePresenter.toInstance(instance, { unknownVersion: "unknown version" }, 0);
        expect(shown.folder).toBe("%LOCALAPPDATA%\\amml\\instances\\1.26.60.05");
        expect(InstancePresenter.shownFolder("D:\\games\\amml")).toBe("D:\\games\\amml");
    });

    it("words a missing AMML version, a running game and the update count", () => {
        const now = 10 * 3_600_000;
        const running = InstancePresenter.toInstance({ ...instance, runningSince: now - 1000 }, { unknownVersion: "unknown version" }, now);
        expect(running.ammlVersion).toBe("unknown version");
        expect(running.running).toBe(true);
        expect(running.updateCount).toBe(1);
        expect(running.activity.startsWith("running since ")).toBe(true);
    });
});

describe("ModPresenter", () => {
    const notes = ModNoteTextSet.build();

    it("counts core mods from the view origin", () => {
        const views = [
            mod({ id: "alaydriem/amml", title: null, origin: "core", registryBase: null }),
            mod({ id: "@amml/modmanager", origin: "core" }),
            mod({}),
        ];
        const mods = views.map((view) => ModPresenter.toInstalled(view, [official], notes));

        expect(mods.map((held) => held.origin)).toEqual(["core", "core", "registry"]);
        expect(mods.filter((held) => held.origin === "core")).toHaveLength(2);
        expect(mods[0]?.title).toBe("AMML loader");
        expect(mods[1]?.note).toBe("core, always on");
        expect(mods[2]?.registryName).toBe("Official");
        expect(mods[2]?.note).toBe("official");
    });

    it("shows a local mod as a url mod with the note local and hides an available version nobody lists", () => {
        const local = ModPresenter.toInstalled(
            mod({ origin: "local", registryBase: null, update: "unknown", availableVersion: null }),
            [official],
            notes,
        );
        expect(local.origin).toBe("url");
        expect(local.note).toBe("local");

        const unlisted = ModPresenter.toInstalled(mod({ update: "notListed", availableVersion: "9.9.9" }), [official], notes);
        expect(unlisted.availableVersion).toBeUndefined();
        expect(ModPresenter.toInstalled(mod({}), [official], notes).availableVersion).toBe("1.1.0");
    });
});

describe("RegistryPresenter", () => {
    it("words the status, the address and the age of a registry", () => {
        const now = 1_000_000 + 4 * 60_000;
        const shown = RegistryPresenter.toSource(official, now);
        expect(shown).toMatchObject({
            id: "abc",
            name: "Official",
            url: "alaydriem.github.io/amml-registry/",
            builtIn: true,
            modCount: 12,
            status: "ok",
            fetched: "4 min ago",
        });
        expect(RegistryPresenter.toSource({ ...official, status: "stale" }, now).status).toBe("ok");
        expect(RegistryPresenter.toSource({ ...official, status: "broken" }, now).status).toBe("failed");
        const unknown = RegistryPresenter.toSource({ ...official, name: null, modCount: null, fetched: null, status: "offline" }, now);
        expect(unknown.name).toBe("alaydriem.github.io/amml-registry/");
        expect(unknown.modCount).toBeUndefined();
        expect(unknown.fetched).toBeUndefined();
    });
});

describe("CatalogPresenter", () => {
    const layout: CatalogLayoutView = {
        registryId: "r",
        api: 2,
        pageSize: 4,
        total: 10,
        groups: [
            { id: "A", label: "A", start: 0, total: 6 },
            { id: "B", label: "B", start: 6, total: 4 },
        ],
    };

    it("builds groups whose loaded rows start at the first loaded page", () => {
        const page = (n: number) => [0, 1, 2, 3].map((i) => card(`p${n}-${i}`));
        const groups = CatalogPresenter.toGroups(layout, new Map([[2, page(2)], [3, page(3)]]));

        expect(groups[0]?.rows.map((row) => row.id)).toEqual(["p2-0", "p2-1"]);
        expect(groups[0]?.offset).toBe(4);
        expect(groups[0]?.start).toBe(0);
        expect(groups[0]?.total).toBe(6);
        expect(groups[1]?.rows.map((row) => row.id)).toEqual(["p2-2", "p2-3", "p3-0", "p3-1"]);
        expect(groups[1]?.offset).toBe(6);
    });

    it("gives a group with no loaded page an empty run at its start", () => {
        const groups = CatalogPresenter.toGroups(layout, new Map());
        expect(groups.map((group) => [group.rows.length, group.offset])).toEqual([[0, 0], [0, 6]]);
    });

    it("stops a run at the first page that is missing", () => {
        const one = [0, 1, 2, 3].map((i) => card(`a${i}`));
        const three = [0, 1, 2, 3].map((i) => card(`c${i}`));
        const groups = CatalogPresenter.toGroups(layout, new Map([[1, one], [3, three]]));

        expect(groups[0]?.rows.map((row) => row.id)).toEqual(["a0", "a1", "a2", "a3"]);
        expect(groups[0]?.offset).toBe(0);
        expect(groups[1]?.rows.map((row) => row.id)).toEqual(["c0", "c1"]);
        expect(groups[1]?.offset).toBe(8);
    });

    it("keeps the state a card has against the instance and gives it an icon or a swatch", () => {
        const view: CardView = {
            id: "@a/b",
            registryId: "r",
            title: "B",
            summary: "s",
            tags: ["t"],
            version: "2.0.0",
            size: 5,
            iconUrl: "https://r.test/icons/b.png",
            state: "update",
            updated: "2026-10-01",
        };
        const shown = CatalogPresenter.toCard(view);
        expect(shown.state).toBe("update");
        expect(shown.icon).toBe("https://r.test/icons/b.png");
        expect(CatalogPresenter.toCard({ ...view, iconUrl: null }).icon).toBeUndefined();
        expect(shown.swatch).toBe(SwatchRules.forName("@a/b"));
    });
});

describe("CheckPresenter", () => {
    const check = (id: CheckView["id"], state: CheckView["state"]): CheckView => ({
        id,
        state,
        package: null,
        version: null,
        freeBytes: null,
        drive: null,
        action: null,
        elevated: false,
    });

    it("lets the player go on when the Store game and Gaming Services are ok", () => {
        expect(CheckPresenter.canContinue([check("store", "ok"), check("gaming-services", "ok"), check("gameinput", "missing")])).toBe(true);
        expect(CheckPresenter.canContinue([check("store", "ok"), check("gaming-services", "missing")])).toBe(false);
        expect(CheckPresenter.canContinue([check("store", "warn"), check("gaming-services", "ok")])).toBe(false);
    });

    it("marks a missing preview app as optional and any other trouble as needing attention", () => {
        expect(CheckPresenter.toSystemCheck(check("preview", "missing")).severity).toBe("muted");
        expect(CheckPresenter.toSystemCheck(check("store", "missing")).severity).toBe("warn");
        expect(CheckPresenter.toSystemCheck(check("disk", "warn")).severity).toBe("warn");
        expect(CheckPresenter.toSystemCheck(check("disk", "ok")).severity).toBe("ok");
    });
});

describe("FaultPresenter", () => {
    it("shows the faults the app raises itself as the general kit kind and keeps its own kind", () => {
        const busy = FaultPresenter.present({ kind: "busy", code: null, values: {}, restored: false });
        expect(busy.kind).toBe("busy");
        expect(busy.fault.kind).toBe("operation-failed");
        expect(busy.fault.severity).toBe("warn");
        expect(busy.text.action).toBe("Show progress");
    });

    it("keeps the code, picks a severity and makes retryable faults the primary action", () => {
        const refused = FaultPresenter.present({ kind: "deploy-refused", code: "0x80073CF3", values: { version: "1.26.51.1" }, restored: true });
        expect(refused.fault.code).toBe("0x80073CF3");
        expect(refused.fault.severity).toBe("fault");
        expect(refused.fault.primaryAction).toBe(false);
        expect(FaultPresenter.present({ kind: "download-failed", code: null, values: {}, restored: false }).fault.primaryAction).toBe(true);
        expect(FaultPresenter.present({ kind: "install-interrupted", code: null, values: { version: "1" }, restored: false }).fault.severity).toBe("info");
        expect(FaultPresenter.present({ kind: "disk-space", code: null, values: {}, restored: false }).fault.severity).toBe("warn");
    });
});

describe("local AMML build", () => {
    const notes = ModNoteTextSet.build();

    it("local_build_row_gets_available_version_and_badge", () => {
        const view = mod({
            id: ModPresenter.AMML_ID,
            origin: "core",
            installedVersion: "26.60.5+local",
            availableVersion: "26.60.5",
            update: "localBuild",
        });

        const row = ModPresenter.toInstalled(view, [official], notes);

        expect(row.availableVersion).toBe("26.60.5");
        expect(row.localBuild).toBe(true);
    });

    it("instance_row_gets_local_flag", () => {
        const local = InstancePresenter.toInstance(
            { ...instance, ammlVersion: "26.60.5+local", ammlLocal: true },
            { unknownVersion: "?" },
            0,
        );
        const release = InstancePresenter.toInstance({ ...instance, ammlVersion: "26.60.5" }, { unknownVersion: "?" }, 0);

        expect(local.ammlLocal).toBe(true);
        expect(release.ammlLocal).toBe(false);
    });
});
