export const worksiteDatasets = ['worksite_workers', 'worksite_minutes'];

export function worksiteMetrics(raw, q) {
  const sites = raw.worksite_cohorts ?? [];
  const site = q.worksite_id === undefined && sites.length === 1
    ? sites[0] : sites.find(s => s.id === q.worksite_id);
  if (!site) {
    if (!sites.length) throw new Error('This result has no placed worksite with its own modeled job stock.');
    const choices = sites.map(({ id, name, destination_cut }) => ({ id, name, destination_cut }));
    throw new Error(`Select one worksite from this result: ${JSON.stringify(choices)}.`);
  }
  if (q.state !== 'scenario') throw new Error('This new worksite has only a scenario cohort. Use state scenario.');
  const time = q.data === 'worksite_minutes';
  if (time ? q.weight !== 'assigned_workers' || q.aggregate === 'sum'
    : q.weight !== 'equal' && q.weight !== 'value') {
    throw new Error('Worksite minutes must be weighted by assigned workers. Use sum or value-weighted share for worker counts.');
  }
  return {
    worksite: { id: site.id, name: site.name, building_id: site.building_id, destination_cut: site.destination_cut, job_stock: site.job_stock },
    rows: site.rows.map(row => ({
      ...row, destination_cut: site.destination_cut, scenario: time ? row.minutes : row.workers,
    })),
    dimensions: ['origin_cut', 'destination_cut', 'mode', 'residence'],
    unit: time ? 'modeled minutes' : 'modeled workers',
    rowUnit: 'worksite origin-by-mode rows',
    population: 'Synthetic weekday workers assigned to this exact scenario worksite.',
    scope: time ? `${site.scope} ${site.time_scope}` : site.scope,
  };
}
