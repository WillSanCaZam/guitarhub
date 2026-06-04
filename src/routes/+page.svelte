<script>
  import ProductCard from '$lib/components/ProductCard.svelte';
  import ProductDetail from '$lib/components/ProductDetail.svelte';
  import PriceChart from '$lib/components/PriceChart.svelte';
  import PriceBadge from '$lib/components/PriceBadge.svelte';
  import Settings from '$lib/components/Settings.svelte';

  const demoProducts = [
    {
      sku: 'FEN-STRAT-001',
      name: 'Fender Player Stratocaster MN TF',
      brand: 'Fender',
      price: 849.99,
      currency: 'USD',
      image_url: 'https://example.com/images/strat-001.jpg'
    },
    {
      sku: 'GIBS-LES-002',
      name: 'Gibson Les Paul Standard 60s',
      brand: 'Gibson',
      price: 2499.00,
      currency: 'USD',
      image_url: 'https://example.com/images/lespaul-002.jpg'
    },
    {
      sku: 'IBAN-RG-003',
      name: 'Ibanez RG550 Genesis Collection',
      brand: 'Ibanez',
      price: 1099.99,
      currency: 'USD',
      image_url: 'https://example.com/images/rg550-003.jpg'
    }
  ];
</script>

<div class="page">
  <header class="page-header">
    <h1>GuitarHub</h1>
    <p class="subtitle">Price tracker for guitar gear</p>
  </header>

  <section class="product-grid">
    <h2>Tracked Products</h2>
    <div class="grid">
      {#each demoProducts as product}
        <ProductCard {product} />
      {/each}
    </div>
  </section>

  <section class="detail-section">
    <h2>Product Detail</h2>
    <ProductDetail product={demoProducts[0]} />
  </section>

  <section class="chart-section">
    <h2>Price History</h2>
    <PriceChart sku={demoProducts[0].sku} windowDays={90} />
  </section>

  <section class="badge-demo">
    <h2>Price Badge Examples</h2>
    <p>
      <PriceBadge level="green" pct={5} />
      <PriceBadge level="amber" pct={12} />
    </p>
  </section>

  <section class="settings-section">
    <Settings />
  </section>
</div>

<style>
  .page {
    max-width: 960px;
    margin: 0 auto;
    padding: 24px 16px;
  }
  .page-header {
    margin-bottom: 32px;
  }
  .page-header h1 {
    margin: 0;
    font-size: 2rem;
  }
  .subtitle {
    color: #666;
    margin: 4px 0 0;
  }
  section {
    margin-bottom: 40px;
  }
  section h2 {
    font-size: 1.25rem;
    margin: 0 0 16px;
    border-bottom: 1px solid #ddd;
    padding-bottom: 8px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }
  .badge-demo p {
    display: flex;
    gap: 8px;
    align-items: center;
  }
</style>
