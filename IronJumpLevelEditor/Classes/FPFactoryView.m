//
//  FPFactoryView.m
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  For license see LICENSE.TXT
//

#import "FPFactoryView.h"

@implementation FPFactoryView

- (id)initWithCoder:(NSCoder *)coder
{
	self = [super initWithCoder:coder];
    if (self) 
	{
		[self setupSharedContext];
		glClearColor(55.0f / 255.0f, 60.0f / 255.0f, 89.0f / 255.0f, 1.0f);
		glEnable(GL_TEXTURE_2D);
		glEnable(GL_BLEND);
		glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }
    return self;
}

- (void)dealloc
{
	[super dealloc];
}

- (BOOL)isFlipped
{
	return YES;
}

- (void)reshape
{
	[self setNeedsDisplay:YES];
}

- (void)mouseDown:(NSEvent *)theEvent
{
	[dataSource setActiveFactory:nil];
	
	CGPoint location = [self locationFromNSEvent:theEvent];
	CGRect rect = CGRectMake(8, 5, 32, 32);
	
	for (id factory in [dataSource factories])
	{
		FPTexture *texture = [factory loadTextureIfNeeded];
		rect.size.width = texture.width;
		rect.size.height = texture.height;
		
		if (CGRectContainsPoint(rect, location))
		{
			[dataSource setActiveFactory:factory];
			[self setNeedsDisplay:YES];
			return;
		}
		
		rect.origin.y += texture.height + 5;
	}
	
	[self setNeedsDisplay:YES];
}

- (void)drawRect:(NSRect)dirtyRect 
{
	[self reshapeFlippedOrtho2D];
	
	glClear(GL_COLOR_BUFFER_BIT);
	CGRect rect = CGRectMake(8, 5, 32, 32);
	
	for (id factory in [dataSource factories])
	{
		FPTexture *texture = [factory loadTextureIfNeeded];
		rect.size.width = texture.width;
		rect.size.height = texture.height;
		[texture drawAtPoint:rect.origin];
		
		if (factory == [dataSource activeFactory])
		{
			glDisable(GL_TEXTURE_2D);
			glColor4f(1, 1, 1, 0.8f);
			glLineWidth(2.0f);
			glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
			glRectf(rect.origin.x, rect.origin.y, rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);
			glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
			glColor4f(1, 1, 1, 1);
		}		
		
		rect.origin.y += texture.height + 5;
	}
	
	[[self openGLContext] flushBuffer];
}

@end
