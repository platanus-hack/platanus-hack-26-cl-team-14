import { randomUUID } from 'node:crypto';
import { renameSync, rmSync, writeFileSync } from 'node:fs';

const displayFile = process.env.SANTIAGO_RESULT_DISPLAY_FILE;
const number = new Intl.NumberFormat('es-CL', { maximumFractionDigits: 2 });
const stateNames = { base: 'Base', scenario: 'Escenario', delta: 'Cambio' };
const titles = {
  metro_entries: 'Entradas al Metro',
  commute_flows: 'Viajes al trabajo',
  route_minutes: 'Tiempo por ruta',
  shopping_trips: 'Viajes de compras',
  shopping_site_total: 'Viajes de compras al lugar',
  shopping_site_origins: 'Origen de las compras',
  shopping_site_modes: 'Modo de viaje para compras',
  shopping_site_departures: 'Hora de salida para compras',
  worksite_workers: 'Trabajadores del lugar',
  worksite_minutes: 'Tiempo de viaje al trabajo',
};

export const resultStatusTool = displayFile && {
  name: 'show_result_status',
  description: 'Show a proposal awaiting confirmation or explain an unavailable metric beside the map. This only updates the UI. It cannot run an experiment or display a numeric result.',
  inputSchema: {
    type: 'object', additionalProperties: false, required: ['cards'],
    properties: {
      cards: {
        type: 'array', minItems: 1, maxItems: 3,
        items: {
          type: 'object', additionalProperties: false, required: ['title', 'status', 'scope'],
          properties: {
            title: { type: 'string', minLength: 1, maxLength: 100 },
            status: { enum: ['pending', 'unavailable'] },
            scope: { type: 'string', minLength: 1, maxLength: 600 },
          },
        },
      },
    },
  },
};

function publish(cards) {
  if (!displayFile) return false;
  const receipt_id = randomUUID();
  const temporary = `${displayFile}.${receipt_id}.tmp`;
  try {
    writeFileSync(temporary, JSON.stringify({ receipt_id, cards }), { mode: 0o600 });
    renameSync(temporary, displayFile);
    return true;
  } catch (error) {
    try { rmSync(temporary, { force: true }); } catch { /* Preserve the tool reply. */ }
    process.stderr.write(`Metric display publication failed: ${error.code ?? error.message}\n`);
    return false;
  }
}

export function publishResultStatus({ cards }) {
  if (!Array.isArray(cards) || cards.length < 1 || cards.length > 3 ||
      cards.some(card => !card || !['pending', 'unavailable'].includes(card.status) ||
        typeof card.title !== 'string' || !card.title.trim() || card.title.length > 100 ||
        typeof card.scope !== 'string' || !card.scope.trim() || card.scope.length > 600)) {
    throw new Error('Provide one to three pending or unavailable cards with a title and scope.');
  }
  const displayed = publish(cards.map(({ title, status, scope }) => ({ title, status, scope, source: 'Consulta' })));
  return { displayed };
}

function cell(value) {
  if (Number.isFinite(value)) return number.format(value);
  if (Number.isFinite(value?.percent)) {
    return `${number.format(value.percent)} % (${number.format(value.numerator)} / ${number.format(value.denominator)})`;
  }
  return '—';
}

function groupLabel(group = {}, comunaNames = {}) {
  return Object.entries(group).map(([key, value]) => {
    if (key === 'half_hour') {
      return `${String(Math.floor(value / 2)).padStart(2, '0')}:${value % 2 ? '30' : '00'}`;
    }
    return key === 'origin_cut' || key === 'destination_cut' ? comunaNames[value] ?? value : value;
  }).join(' · ');
}

function displayRows(result, states, comunaNames) {
  return result.results.flatMap(({ group, value, selected_workers, covered_workers }) => {
    const label = groupLabel(group, comunaNames);
    const coverage = label && Number.isFinite(selected_workers) && Number.isFinite(covered_workers)
      ? `${cell(covered_workers)} de ${cell(selected_workers)} con datos` : '';
    const values = states.length === 1 ? [value] : states.map(state => value?.[state]);
    const bins = values.find(Array.isArray);
    if (!bins) return [{ label: [label, coverage].filter(Boolean).join(' · ') || 'Valor', values: values.map(cell) }];
    return bins.map(({ min_inclusive: min, max_exclusive: max }, index) => {
      const range = min === null ? `< ${cell(max)}` : max === null ? `≥ ${cell(min)}` : `${cell(min)} a < ${cell(max)}`;
      return { label: [label, range, index === 0 && coverage].filter(Boolean).join(' · '), values: values.map(value => cell(value?.[index]?.count)) };
    });
  });
}

function coverageText(c) {
  const parts = [];
  if (Number.isFinite(c.selected_workers)) {
    parts.push(`${cell(c.covered_workers)} de ${cell(c.selected_workers)} trabajadores cubiertos.`);
    if (c.missing_workers) parts.push(`${cell(c.missing_workers)} sin datos.`);
  } else if (Number.isFinite(c.selected_rows)) {
    parts.push(`${cell(c.valid_rows)} de ${cell(c.selected_rows)} filas seleccionadas con datos.`);
    if (c.missing_or_unreachable_rows) parts.push(`${cell(c.missing_or_unreachable_rows)} sin datos o sin ruta.`);
  }
  if (Number.isFinite(c.observed_public_commuters_in_scope)) {
    parts.push(`${cell(c.commuters_with_paired_times)} de ${cell(c.observed_public_commuters_in_scope)} trabajadores censales con tiempos válidos.`);
  }
  if (c.may_be_truncated) parts.push('La fuente incluye solo una parte de las filas.');
  if (c.groups_returned < c.groups) parts.push(`Se muestran ${cell(c.groups_returned)} de ${cell(c.groups)} grupos.`);
  return parts.join(' ');
}

export function publishMetricResult(args, result, { sourceTitle, comunaNames, error } = {}) {
  if (!displayFile) return;
  const source = args.source?.type === 'replay' ? `Replay · ${sourceTitle ?? args.source.id ?? ''}` : 'Escenario actual';
  const title = [titles[args.data] ?? 'Métrica del experimento', result?.worksite?.name].filter(Boolean).join(' · ');
  if (!result) {
    publish([{ title, source, status: 'unavailable', coverage: error }]);
    return;
  }
  const states = result.state === 'compare' ? ['base', 'scenario'] : [result.state];
  const c = result.coverage ?? {};
  const rows = displayRows(result, states, comunaNames);
  const available = c.valid_rows !== 0 && rows.some(row => row.values.some(value => value !== '—'));
  const incomplete = c.missing_or_unreachable_rows || c.missing_workers || c.pairs_without_positive_public_weight ||
    c.may_be_truncated || c.groups_returned < c.groups || c.commuters_with_paired_times < c.observed_public_commuters_in_scope;
  const unit = result.aggregate === 'histogram' ? `${result.unit} · intervalos en ${result.measured_unit}` : result.unit;
  const scope = [result.scope, result.data === 'route_minutes' && result.weighting].filter(Boolean).join(' ');
  publish([{
    title, source, status: !available ? 'unavailable' : incomplete ? 'partial' : 'result', unit, scope,
    coverage: coverageText(c), columns: states.map(state => result.state_labels?.[state] ?? stateNames[state]),
    rows: available ? rows : [],
  }]);
}

export function publishExperimentResult(raw, { sourceTitle } = {}) {
  if (!displayFile) return;
  const source = sourceTitle ? `Replay · ${sourceTitle}` : 'Escenario actual';
  const missing = raw.not_modeled ?? [];
  const cards = (raw.worksite_cohorts ?? []).map(site => ({
    title: site.name ?? 'Trabajadores del lugar', source, status: 'result', unit: site.unit,
    scope: site.scope, columns: ['Trabajadores'],
    rows: [
      { label: 'Asignados al lugar', values: [cell(site.assigned_workers)] },
      { label: 'Desde otras comunas', values: [cell(site.outside_workers)] },
    ],
  }));
  if (!cards.length && (!missing.length || raw.applied?.length) && Number.isFinite(raw.summary?.workers_moved)) {
    cards.push({
      title: 'Cambio de lugar de trabajo', source, status: 'result', unit: 'trabajadores modelados',
      scope: 'Día hábil típico. Incluye cambios de lugar de trabajo dentro de una misma comuna.',
      rows: [{ label: 'Trabajadores reasignados', values: [cell(raw.summary.workers_moved)] }],
    });
  }
  if (missing.length) {
    cards.push({
      title: 'Cambios sin resultado', source, status: 'unavailable',
      coverage: missing.map(change => change.reason).filter(Boolean).join(' ') || 'El motor no pudo aplicar parte de la solicitud.',
    });
  }
  if (cards.length) publish(cards);
}
