import { Component, HostListener } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterModule, NavigationEnd, UrlTree } from '@angular/router';
import {AgentPreview} from '../agent-preview/agent-preview';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [CommonModule, RouterModule, AgentPreview],
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss']
})
export class HomeComponent {
  constructor(private router: Router) {
    // Listen for navigation events and handle fragments consistently
    this.router.events.subscribe(event => {
      if (event instanceof NavigationEnd) {
        const tree: UrlTree = this.router.parseUrl(this.router.url);
        if (tree.fragment) {
          this.scrollToSection(tree.fragment);
        }
      }
    });
  }

  @HostListener('window:hashchange')
  onHashChange() {
    const fragment = window.location.hash.replace('#', '');
    if (fragment) {
      this.scrollToSection(fragment);
    }
  }

  private scrollToSection(id: string) {
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }
}
